use crate::config::AppState;
use crate::web::session_extractor::extract_session_data;
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    http::HeaderMap,
};

pub async fn server_list(
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };
    
    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };
    
    let manageable_guilds = user.get_manageable_guilds();
    
    let mut guilds_html = String::new();
    for guild in manageable_guilds {
        let icon_url = if let Some(icon) = &guild.icon {
            format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon)
        } else {
            "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
        };
        
        guilds_html.push_str(&format!(
            r#"
            <div class="server-card" onclick="location.href='/dashboard/{}'">
                <img src="{}" alt="{}" class="server-icon">
                <div class="server-info">
                    <h3>{}</h3>
                    <p>{} permission</p>
                </div>
            </div>
            "#,
            guild.id,
            icon_url,
            guild.name,
            guild.name,
            if guild.owner { "Owner" } else { "Manage Roles" }
        ));
    }
    
    if guilds_html.is_empty() {
        let has_guilds = !user.guilds.is_empty();
        guilds_html = if has_guilds {
            r#"
            <div class="no-servers">
                <h3>No manageable servers found</h3>
                <p>You need "Manage Roles" permission in a server to configure self-roles.</p>
                <p><a href="https://discord.com/developers/applications" target="_blank">Invite the bot to your server</a></p>
            </div>
            "#.to_string()
        } else {
            r#"
            <div class="no-servers">
                <h3>Guilds could not be loaded</h3>
                <p>There was an error loading your Discord servers. This might be a temporary issue.</p>
                <p>You are successfully logged in as a user, but guild data couldn't be retrieved.</p>
                <p><a href="/auth/logout">Logout and try again</a></p>
            </div>
            "#.to_string()
        };
    }
    
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Clouder Bot Dashboard</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                body {{
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    margin: 0;
                    padding: 20px;
                    min-height: 100vh;
                }}
                .container {{
                    max-width: 1200px;
                    margin: 0 auto;
                }}
                .header {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 30px;
                    margin-bottom: 30px;
                    text-align: center;
                    color: white;
                }}
                .header h1 {{
                    margin: 0 0 10px 0;
                    font-size: 2.5em;
                    font-weight: 300;
                }}
                .user-info {{
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 15px;
                    margin-top: 20px;
                }}
                .user-avatar {{
                    width: 50px;
                    height: 50px;
                    border-radius: 50%;
                    border: 3px solid rgba(255, 255, 255, 0.3);
                }}
                .servers-grid {{
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
                    gap: 20px;
                    margin-top: 20px;
                }}
                .server-card {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 25px;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    display: flex;
                    align-items: center;
                    gap: 20px;
                    color: white;
                }}
                .server-card:hover {{
                    transform: translateY(-5px);
                    background: rgba(255, 255, 255, 0.2);
                    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
                }}
                .server-icon {{
                    width: 60px;
                    height: 60px;
                    border-radius: 50%;
                    border: 3px solid rgba(255, 255, 255, 0.3);
                }}
                .server-info h3 {{
                    margin: 0 0 5px 0;
                    font-size: 1.3em;
                }}
                .server-info p {{
                    margin: 0;
                    opacity: 0.8;
                    font-size: 0.9em;
                }}
                .no-servers {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 40px;
                    text-align: center;
                    color: white;
                }}
                .no-servers h3 {{
                    margin-top: 0;
                    font-size: 1.5em;
                }}
                .no-servers a {{
                    color: #ffffff;
                    text-decoration: underline;
                }}
                .logout-btn {{
                    position: absolute;
                    top: 20px;
                    right: 20px;
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                    padding: 10px 20px;
                    border: none;
                    border-radius: 25px;
                    cursor: pointer;
                    text-decoration: none;
                    transition: background 0.3s ease;
                }}
                .logout-btn:hover {{
                    background: rgba(255, 255, 255, 0.3);
                }}
            </style>
        </head>
        <body>
            <a href="/auth/logout" class="logout-btn">Logout</a>
            <div class="container">
                <div class="header">
                    <h1>Clouder Bot Dashboard</h1>
                    <p>Select a server to configure self-roles</p>
                    <div class="user-info">
                        <img src="{}" alt="{}" class="user-avatar">
                        <span>Welcome, {}!</span>
                    </div>
                </div>
                <div class="servers-grid">
                    {}
                </div>
            </div>
        </body>
        </html>
        "#,
        if let Some(avatar) = &user.user.avatar {
            format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.user.id, avatar)
        } else {
            "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
        },
        user.user.username,
        user.user.username,
        guilds_html
    );
    
    Ok(Html(html))
}

pub async fn guild_dashboard(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };
    
    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };
    
    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }
    
    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();
    
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Dashboard - {}</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                body {{
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    margin: 0;
                    padding: 20px;
                    min-height: 100vh;
                }}
                .container {{
                    max-width: 1000px;
                    margin: 0 auto;
                }}
                .header {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 30px;
                    margin-bottom: 30px;
                    color: white;
                    display: flex;
                    align-items: center;
                    gap: 20px;
                }}
                .guild-icon {{
                    width: 80px;
                    height: 80px;
                    border-radius: 50%;
                    border: 3px solid rgba(255, 255, 255, 0.3);
                }}
                .header-info h1 {{
                    margin: 0 0 5px 0;
                    font-size: 2em;
                }}
                .header-info p {{
                    margin: 0;
                    opacity: 0.8;
                }}
                .back-btn {{
                    position: absolute;
                    top: 20px;
                    left: 20px;
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                    padding: 10px 20px;
                    border: none;
                    border-radius: 25px;
                    cursor: pointer;
                    text-decoration: none;
                    transition: background 0.3s ease;
                }}
                .back-btn:hover {{
                    background: rgba(255, 255, 255, 0.3);
                }}
                .logout-btn {{
                    position: absolute;
                    top: 20px;
                    right: 20px;
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                    padding: 10px 20px;
                    border: none;
                    border-radius: 25px;
                    cursor: pointer;
                    text-decoration: none;
                    transition: background 0.3s ease;
                }}
                .logout-btn:hover {{
                    background: rgba(255, 255, 255, 0.3);
                }}
                .features-grid {{
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                    gap: 25px;
                }}
                .feature-card {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 30px;
                    color: white;
                    cursor: pointer;
                    transition: all 0.3s ease;
                }}
                .feature-card:hover {{
                    transform: translateY(-5px);
                    background: rgba(255, 255, 255, 0.2);
                    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
                }}
                .feature-icon {{
                    font-size: 3em;
                    margin-bottom: 15px;
                }}
                .feature-card h3 {{
                    margin: 0 0 10px 0;
                    font-size: 1.4em;
                }}
                .feature-card p {{
                    margin: 0;
                    opacity: 0.9;
                    line-height: 1.6;
                }}
            </style>
        </head>
        <body>
            <a href="/" class="back-btn">← Back to Servers</a>
            <a href="/auth/logout" class="logout-btn">Logout</a>
            <div class="container">
                <div class="header">
                    <img src="{}" alt="{}" class="guild-icon">
                    <div class="header-info">
                        <h1>{}</h1>
                        <p>Configure bot features for this server</p>
                    </div>
                </div>
                <div class="features-grid">
                    <div class="feature-card" onclick="location.href='/dashboard/{}/selfroles'">
                        <div class="feature-icon">🎭</div>
                        <h3>Self Roles</h3>
                        <p>Create interactive role assignment messages. Let members choose their own roles by clicking buttons.</p>
                    </div>
                    <div class="feature-card" style="opacity: 0.6; cursor: not-allowed;">
                        <div class="feature-icon">⚙️</div>
                        <h3>Server Settings</h3>
                        <p>Configure timezone, prefixes, and other server-specific settings. (Coming Soon)</p>
                    </div>
                    <div class="feature-card" style="opacity: 0.6; cursor: not-allowed;">
                        <div class="feature-icon">📊</div>
                        <h3>Analytics</h3>
                        <p>View usage statistics and insights about your server's bot activity. (Coming Soon)</p>
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#,
        guild.name,
        if let Some(icon) = &guild.icon {
            format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon)
        } else {
            "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
        },
        guild.name,
        guild.name,
        guild_id
    );
    
    Ok(Html(html))
}

pub async fn selfroles_dashboard(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };
    
    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };
    
    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }
    
    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();
    
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Self-Roles - {}</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                body {{
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    margin: 0;
                    padding: 20px;
                    min-height: 100vh;
                }}
                .container {{
                    max-width: 1200px;
                    margin: 0 auto;
                }}
                .header {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 30px;
                    margin-bottom: 30px;
                    color: white;
                    text-align: center;
                }}
                .header h1 {{
                    margin: 0 0 10px 0;
                    font-size: 2.2em;
                }}
                .main-content {{
                    display: grid;
                    grid-template-columns: 1fr 400px;
                    gap: 30px;
                }}
                .form-section {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 30px;
                    color: white;
                }}
                .preview-section {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 30px;
                    color: white;
                    position: sticky;
                    top: 20px;
                    height: fit-content;
                }}
                .form-group {{
                    margin-bottom: 20px;
                }}
                label {{
                    display: block;
                    margin-bottom: 8px;
                    font-weight: 500;
                    font-size: 0.95em;
                }}
                input, textarea, select {{
                    width: 100%;
                    padding: 12px;
                    border: 2px solid rgba(255, 255, 255, 0.2);
                    border-radius: 8px;
                    background: rgba(255, 255, 255, 0.1);
                    color: white;
                    font-size: 14px;
                    box-sizing: border-box;
                }}
                input::placeholder, textarea::placeholder {{
                    color: rgba(255, 255, 255, 0.6);
                }}
                input:focus, textarea:focus, select:focus {{
                    outline: none;
                    border-color: rgba(255, 255, 255, 0.5);
                    background: rgba(255, 255, 255, 0.15);
                }}
                textarea {{
                    height: 100px;
                    resize: vertical;
                }}
                .selection-type {{
                    display: flex;
                    gap: 20px;
                    margin-bottom: 20px;
                }}
                .radio-group {{
                    display: flex;
                    align-items: center;
                    gap: 8px;
                }}
                .radio-group input[type="radio"] {{
                    width: auto;
                    margin: 0;
                }}
                .roles-section {{
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 10px;
                    padding: 20px;
                    margin-bottom: 20px;
                }}
                .role-item {{
                    display: flex;
                    align-items: center;
                    padding: 10px;
                    margin-bottom: 10px;
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 8px;
                    gap: 15px;
                }}
                .role-checkbox {{
                    width: auto !important;
                    margin: 0 !important;
                }}
                .emoji-input {{
                    width: 80px !important;
                    text-align: center;
                }}
                .role-name {{
                    flex: 1;
                    font-weight: 500;
                }}
                .preview-embed {{
                    background: rgba(0, 0, 0, 0.3);
                    border-left: 4px solid #667eea;
                    border-radius: 8px;
                    padding: 20px;
                    margin-bottom: 20px;
                }}
                .preview-title {{
                    font-weight: bold;
                    font-size: 1.1em;
                    margin-bottom: 10px;
                }}
                .preview-body {{
                    opacity: 0.9;
                    line-height: 1.5;
                }}
                .preview-buttons {{
                    display: flex;
                    flex-wrap: wrap;
                    gap: 10px;
                }}
                .preview-button {{
                    background: rgba(88, 101, 242, 0.8);
                    color: white;
                    padding: 8px 16px;
                    border-radius: 6px;
                    font-size: 14px;
                    border: none;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    gap: 6px;
                }}
                .btn {{
                    background: linear-gradient(45deg, #667eea, #764ba2);
                    color: white;
                    padding: 15px 30px;
                    border: none;
                    border-radius: 8px;
                    font-size: 16px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    width: 100%;
                }}
                .btn:hover {{
                    transform: translateY(-2px);
                    box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
                }}
                .btn:disabled {{
                    opacity: 0.6;
                    cursor: not-allowed;
                    transform: none;
                }}
                .back-btn {{
                    position: absolute;
                    top: 20px;
                    left: 20px;
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                    padding: 10px 20px;
                    border: none;
                    border-radius: 25px;
                    cursor: pointer;
                    text-decoration: none;
                    transition: background 0.3s ease;
                }}
                .back-btn:hover {{
                    background: rgba(255, 255, 255, 0.3);
                }}
                .loading {{
                    display: none;
                    color: rgba(255, 255, 255, 0.8);
                    font-style: italic;
                }}
                @media (max-width: 768px) {{
                    .main-content {{
                        grid-template-columns: 1fr;
                    }}
                    .preview-section {{
                        position: static;
                    }}
                }}
            </style>
        </head>
        <body>
            <a href="/dashboard/{}" class="back-btn">← Back to Dashboard</a>
            <div class="container">
                <div class="header">
                    <h1>Self-Roles Configuration</h1>
                    <p>Create interactive role assignment messages for {}</p>
                </div>
                <div class="main-content">
                    <div class="form-section">
                        <form id="selfRoleForm">
                            <div class="form-group">
                                <label for="channel">Target Channel:</label>
                                <select id="channel" name="channel_id" required>
                                    <option value="">Loading channels...</option>
                                </select>
                            </div>
                            
                            <div class="form-group">
                                <label for="title">Embed Title:</label>
                                <input type="text" id="title" name="title" placeholder="Choose your roles" required maxlength="256">
                            </div>
                            
                            <div class="form-group">
                                <label for="body">Embed Description:</label>
                                <textarea id="body" name="body" placeholder="Click the buttons below to assign yourself roles..." required maxlength="2048"></textarea>
                            </div>
                            
                            <div class="form-group">
                                <label>Selection Type:</label>
                                <div class="selection-type">
                                    <div class="radio-group">
                                        <input type="radio" id="multiple" name="selection_type" value="multiple" checked>
                                        <label for="multiple">Multiple Selection</label>
                                    </div>
                                    <div class="radio-group">
                                        <input type="radio" id="radio" name="selection_type" value="radio">
                                        <label for="radio">Single Selection</label>
                                    </div>
                                </div>
                            </div>
                            
                            <div class="form-group">
                                <label>Roles:</label>
                                <div class="roles-section">
                                    <div id="rolesList">
                                        <div class="loading">Loading server roles...</div>
                                    </div>
                                </div>
                            </div>
                            
                            <button type="submit" class="btn" id="deployBtn" disabled>
                                Deploy Self-Role Message
                            </button>
                        </form>
                    </div>
                    
                    <div class="preview-section">
                        <h3 style="margin-top: 0;">Live Preview</h3>
                        <div class="preview-embed">
                            <div class="preview-title" id="previewTitle">Choose your roles</div>
                            <div class="preview-body" id="previewBody">Click the buttons below to assign yourself roles...</div>
                        </div>
                        <div class="preview-buttons" id="previewButtons">
                            <!-- Buttons will be generated here -->
                        </div>
                    </div>
                </div>
            </div>
            
            <script>
                const guildId = '{}';
                let channels = [];
                let roles = [];
                
                // Load channels and roles on page load
                document.addEventListener('DOMContentLoaded', async function() {{
                    await Promise.all([loadChannels(), loadRoles()]);
                    updateDeployButton();
                }});
                
                async function loadChannels() {{
                    try {{
                        const response = await fetch(`/api/guild/${{guildId}}/channels`);
                        const data = await response.json();
                        channels = data.channels.filter(ch => ch.type === 0); // Text channels only
                        
                        const channelSelect = document.getElementById('channel');
                        channelSelect.innerHTML = '<option value="">Select a channel...</option>';
                        channels.forEach(channel => {{
                            channelSelect.innerHTML += `<option value="${{channel.id}}">#${{channel.name}}</option>`;
                        }});
                    }} catch (error) {{
                        console.error('Failed to load channels:', error);
                        document.getElementById('channel').innerHTML = '<option value="">Failed to load channels</option>';
                    }}
                }}
                
                async function loadRoles() {{
                    try {{
                        const response = await fetch(`/api/guild/${{guildId}}/roles`);
                        const data = await response.json();
                        roles = data.roles.filter(role => role.name !== '@everyone').sort((a, b) => b.position - a.position);
                        
                        const rolesList = document.getElementById('rolesList');
                        rolesList.innerHTML = '';
                        
                        roles.forEach(role => {{
                            const roleItem = document.createElement('div');
                            roleItem.className = 'role-item';
                            roleItem.innerHTML = `
                                <input type="checkbox" class="role-checkbox" data-role-id="${{role.id}}" onchange="updatePreview()">
                                <input type="text" class="emoji-input" placeholder="🎮" maxlength="2" onchange="updatePreview()">
                                <div class="role-name" style="color: #${{role.color.toString(16).padStart(6, '0')}}">${{role.name}}</div>
                            `;
                            rolesList.appendChild(roleItem);
                        }});
                    }} catch (error) {{
                        console.error('Failed to load roles:', error);
                        document.getElementById('rolesList').innerHTML = '<div>Failed to load roles</div>';
                    }}
                }}
                
                function updatePreview() {{
                    const title = document.getElementById('title').value || 'Choose your roles';
                    const body = document.getElementById('body').value || 'Click the buttons below to assign yourself roles...';
                    
                    document.getElementById('previewTitle').textContent = title;
                    document.getElementById('previewBody').textContent = body;
                    
                    const previewButtons = document.getElementById('previewButtons');
                    previewButtons.innerHTML = '';
                    
                    const checkboxes = document.querySelectorAll('.role-checkbox:checked');
                    checkboxes.forEach(checkbox => {{
                        const roleId = checkbox.dataset.roleId;
                        const role = roles.find(r => r.id === roleId);
                        const emojiInput = checkbox.parentElement.querySelector('.emoji-input');
                        const emoji = emojiInput.value || '📝';
                        
                        if (role) {{
                            const button = document.createElement('button');
                            button.className = 'preview-button';
                            button.innerHTML = `${{emoji}} ${{role.name}}`;
                            previewButtons.appendChild(button);
                        }}
                    }});
                    
                    updateDeployButton();
                }}
                
                function updateDeployButton() {{
                    const channel = document.getElementById('channel').value;
                    const title = document.getElementById('title').value;
                    const body = document.getElementById('body').value;
                    const selectedRoles = document.querySelectorAll('.role-checkbox:checked');
                    
                    const deployBtn = document.getElementById('deployBtn');
                    deployBtn.disabled = !channel || !title || !body || selectedRoles.length === 0;
                }}
                
                // Add event listeners
                document.getElementById('title').addEventListener('input', updatePreview);
                document.getElementById('body').addEventListener('input', updatePreview);
                document.getElementById('channel').addEventListener('change', updateDeployButton);
                document.querySelectorAll('input[name="selection_type"]').forEach(radio => {{
                    radio.addEventListener('change', updatePreview);
                }});
                
                // Handle form submission
                document.getElementById('selfRoleForm').addEventListener('submit', async function(e) {{
                    e.preventDefault();
                    
                    const deployBtn = document.getElementById('deployBtn');
                    deployBtn.disabled = true;
                    deployBtn.textContent = 'Deploying...';
                    
                    const formData = new FormData(this);
                    const selectedRoles = [];
                    
                    document.querySelectorAll('.role-checkbox:checked').forEach(checkbox => {{
                        const roleId = checkbox.dataset.roleId;
                        const emoji = checkbox.parentElement.querySelector('.emoji-input').value || '📝';
                        selectedRoles.push({{ role_id: roleId, emoji: emoji }});
                    }});
                    
                    const payload = {{
                        title: formData.get('title'),
                        body: formData.get('body'),
                        selection_type: formData.get('selection_type'),
                        channel_id: formData.get('channel_id'),
                        roles: selectedRoles
                    }};
                    
                    try {{
                        const response = await fetch(`/api/selfroles/${{guildId}}`, {{
                            method: 'POST',
                            headers: {{
                                'Content-Type': 'application/json',
                            }},
                            body: JSON.stringify(payload)
                        }});
                        
                        const result = await response.json();
                        
                        if (response.ok && result.success) {{
                            alert('Self-role message deployed successfully!');
                            window.location.href = `/dashboard/${{guildId}}`;
                        }} else {{
                            alert('Failed to deploy self-role message: ' + (result.message || 'Unknown error'));
                        }}
                    }} catch (error) {{
                        console.error('Deployment failed:', error);
                        alert('Failed to deploy self-role message. Please try again.');
                    }} finally {{
                        deployBtn.disabled = false;
                        deployBtn.textContent = 'Deploy Self-Role Message';
                    }}
                }});
            </script>
        </body>
        </html>
        "#,
        guild.name,
        guild_id,
        guild.name,
        guild_id
    );
    
    Ok(Html(html))
}