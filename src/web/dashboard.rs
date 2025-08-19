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

pub async fn selfroles_list(
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
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                }}
                .header-info h1 {{
                    margin: 0 0 5px 0;
                    font-size: 2.2em;
                }}
                .header-info p {{
                    margin: 0;
                    opacity: 0.8;
                }}
                .create-btn {{
                    background: linear-gradient(45deg, #667eea, #764ba2);
                    color: white;
                    padding: 12px 24px;
                    border: none;
                    border-radius: 8px;
                    font-size: 16px;
                    font-weight: 500;
                    cursor: pointer;
                    text-decoration: none;
                    display: inline-block;
                    transition: all 0.3s ease;
                }}
                .create-btn:hover {{
                    transform: translateY(-2px);
                    box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
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
                .messages-grid {{
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
                    gap: 20px;
                }}
                .message-card {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 25px;
                    color: white;
                    transition: all 0.3s ease;
                    cursor: pointer;
                }}
                .message-card:hover {{
                    transform: translateY(-5px);
                    background: rgba(255, 255, 255, 0.2);
                    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
                }}
                .message-title {{
                    font-size: 1.3em;
                    font-weight: 600;
                    margin-bottom: 10px;
                    display: -webkit-box;
                    -webkit-line-clamp: 2;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                }}
                .message-body {{
                    opacity: 0.8;
                    margin-bottom: 15px;
                    display: -webkit-box;
                    -webkit-line-clamp: 3;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                    line-height: 1.4;
                }}
                .message-meta {{
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    font-size: 0.9em;
                    opacity: 0.7;
                    margin-bottom: 15px;
                }}
                .message-actions {{
                    display: flex;
                    gap: 10px;
                    margin-top: 15px;
                }}
                .edit-btn, .delete-btn {{
                    padding: 8px 16px;
                    border: none;
                    border-radius: 20px;
                    cursor: pointer;
                    font-size: 0.85em;
                    font-weight: 500;
                    text-decoration: none;
                    display: inline-block;
                    transition: all 0.3s ease;
                }}
                .edit-btn {{
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                }}
                .edit-btn:hover {{
                    background: rgba(255, 255, 255, 0.3);
                }}
                .delete-btn {{
                    background: rgba(220, 53, 69, 0.8);
                    color: white;
                }}
                .delete-btn:hover {{
                    background: rgba(220, 53, 69, 1);
                }}
                .role-count {{
                    background: rgba(255, 255, 255, 0.2);
                    padding: 4px 8px;
                    border-radius: 12px;
                    font-size: 0.8em;
                }}
                .selection-type {{
                    background: rgba(102, 126, 234, 0.3);
                    padding: 4px 8px;
                    border-radius: 12px;
                    font-size: 0.8em;
                }}
                .no-messages {{
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border-radius: 15px;
                    padding: 60px;
                    text-align: center;
                    color: white;
                    grid-column: 1 / -1;
                }}
                .no-messages h3 {{
                    margin-top: 0;
                    font-size: 1.5em;
                    margin-bottom: 15px;
                }}
                .no-messages p {{
                    opacity: 0.8;
                    margin-bottom: 25px;
                }}
                .loading {{
                    text-align: center;
                    color: white;
                    padding: 40px;
                    grid-column: 1 / -1;
                }}
            </style>
        </head>
        <body>
            <a href="/dashboard/{}" class="back-btn">← Back to Dashboard</a>
            <div class="container">
                <div class="header">
                    <div class="header-info">
                        <h1>Self-Roles Messages</h1>
                        <p>Manage interactive role assignment messages for {}</p>
                    </div>
                    <a href="/dashboard/{}/selfroles/new" class="create-btn">+ Create New Message</a>
                </div>
                <div class="messages-grid" id="messagesGrid">
                    <div class="loading">Loading self-role messages...</div>
                </div>
            </div>
            
            <script>
                const guildId = '{}';
                
                document.addEventListener('DOMContentLoaded', async function() {{
                    await loadMessages();
                }});
                
                async function loadMessages() {{
                    try {{
                        const response = await fetch(`/api/selfroles/${{guildId}}`);
                        const data = await response.json();
                        
                        const messagesGrid = document.getElementById('messagesGrid');
                        
                        if (data.success && data.configs.length > 0) {{
                            messagesGrid.innerHTML = '';
                            
                            data.configs.forEach(config => {{
                                const messageCard = document.createElement('div');
                                messageCard.className = 'message-card';
                                
                                const createdDate = new Date(config.created_at).toLocaleDateString();
                                
                                messageCard.innerHTML = `
                                    <div class="message-title">${{config.title}}</div>
                                    <div class="message-body">${{config.body}}</div>
                                    <div class="message-meta">
                                        <span>Created: ${{createdDate}}</span>
                                        <div>
                                            <span class="role-count">${{config.role_count}} roles</span>
                                            <span class="selection-type">${{config.selection_type}}</span>
                                        </div>
                                    </div>
                                    <div class="message-actions">
                                        <a href="/dashboard/${{guildId}}/selfroles/edit/${{config.id}}" class="edit-btn">Edit</a>
                                        <button class="delete-btn" onclick="deleteMessage(event, ${{config.id}}, '${{config.title}}')">Delete</button>
                                    </div>
                                `;
                                
                                messagesGrid.appendChild(messageCard);
                            }});
                        }} else {{
                            messagesGrid.innerHTML = `
                                <div class="no-messages">
                                    <h3>No self-role messages yet</h3>
                                    <p>Create your first interactive role assignment message to get started!</p>
                                    <a href="/dashboard/${{guildId}}/selfroles/new" class="create-btn">+ Create First Message</a>
                                </div>
                            `;
                        }}
                    }} catch (error) {{
                        console.error('Failed to load messages:', error);
                        document.getElementById('messagesGrid').innerHTML = `
                            <div class="no-messages">
                                <h3>Failed to load messages</h3>
                                <p>There was an error loading your self-role messages. Please try refreshing the page.</p>
                            </div>
                        `;
                    }}
                }}
                
                async function deleteMessage(event, configId, title) {{
                    event.stopPropagation();
                    
                    if (!confirm(`Are you sure you want to delete "${{title}}"? This will also remove the message from Discord and cannot be undone.`)) {{
                        return;
                    }}
                    
                    try {{
                        const response = await fetch(`/api/selfroles/${{guildId}}/${{configId}}`, {{
                            method: 'DELETE'
                        }});
                        
                        const result = await response.json();
                        
                        if (response.ok && result.success) {{
                            alert('Self-role message deleted successfully!');
                            await loadMessages(); // Reload the messages list
                        }} else {{
                            alert('Failed to delete self-role message: ' + (result.message || 'Unknown error'));
                        }}
                    }} catch (error) {{
                        console.error('Delete failed:', error);
                        alert('Failed to delete self-role message. Please try again.');
                    }}
                }}
            </script>
        </body>
        </html>
        "#,
        guild.name,
        guild_id,
        guild.name,
        guild_id,
        guild_id
    );
    
    Ok(Html(html))
}

pub async fn selfroles_create(
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
            <title>Create Self-Role Message - {}</title>
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
            <a href="/dashboard/{}/selfroles" class="back-btn">← Back to Self-Roles</a>
            <div class="container">
                <div class="header">
                    <h1>Create Self-Role Message</h1>
                    <p>Create a new interactive role assignment message for {}</p>
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
                                <div style="background: rgba(255, 193, 7, 0.2); border-left: 4px solid #ffc107; padding: 12px; border-radius: 4px; margin-bottom: 15px; color: #fff3cd;">
                                    <strong>ℹ️ Role Hierarchy Notice:</strong> Only roles that are below at least one of the bot's roles in the server hierarchy can be assigned through self-roles. If you don't see a role here, make sure at least one of the bot's roles is positioned above it in Server Settings → Roles.
                                </div>
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
                        
                        if (!data.success) {{
                            // Handle permission or other errors
                            const rolesList = document.getElementById('rolesList');
                            rolesList.innerHTML = `
                                <div style="color: #ffcccb; padding: 15px; background: rgba(220, 53, 69, 0.2); border-radius: 8px; margin-bottom: 15px;">
                                    <strong>⚠️ Error Loading Roles:</strong><br>
                                    ${{data.message || 'Failed to load server roles.'}}
                                    <br><br>
                                    <strong>Common solutions:</strong>
                                    <ul style="margin: 10px 0;">
                                        <li>Ensure the bot is properly added to your server</li>
                                        <li>Make sure the bot has "Manage Roles" permission</li>
                                        <li>Check that the bot's role is above the roles you want to manage</li>
                                        <li>Try re-inviting the bot with proper permissions</li>
                                    </ul>
                                </div>
                            `;
                            return;
                        }}
                        
                        roles = data.roles.filter(role => role.name !== '@everyone').sort((a, b) => b.position - a.position);
                        
                        const rolesList = document.getElementById('rolesList');
                        rolesList.innerHTML = '';
                        
                        if (roles.length === 0) {{
                            rolesList.innerHTML = `
                                <div style="color: #fff3cd; padding: 15px; background: rgba(255, 193, 7, 0.2); border-radius: 8px; margin-bottom: 15px;">
                                    <strong>ℹ️ No Manageable Roles Found</strong><br>
                                    The bot cannot manage any roles in this server. This could be because:
                                    <ul style="margin: 10px 0;">
                                        <li>All server roles are positioned above the bot's highest role</li>
                                        <li>The bot lacks "Manage Roles" permission</li>
                                        <li>No roles have been created in this server yet</li>
                                    </ul>
                                    Please adjust role positions in Server Settings → Roles, or create some roles for the bot to manage.
                                </div>
                            `;
                            return;
                        }}
                        
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
                        document.getElementById('rolesList').innerHTML = `
                            <div style="color: #ffcccb; padding: 15px; background: rgba(220, 53, 69, 0.2); border-radius: 8px;">
                                <strong>❌ Network Error</strong><br>
                                Failed to load roles due to a network error. Please check your connection and try again.
                            </div>
                        `;
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
                            window.location.href = `/dashboard/${{guildId}}/selfroles`;
                        }} else {{
                            alert('Failed to deploy self-role message: ' + (result.message || 'Unknown error'));
                        }}
                    }} catch (error) {{
                        console.error('Deployment failed:', error);
                        alert('Failed to deploy self-role message. Please try again.');
                    }} finally {{
                        deployBtn.disabled = false;
                        deployBtn.textContent = 'Update Self-Role Message';
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

pub async fn selfroles_edit(
    Path((guild_id, config_id)): Path<(String, String)>,
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
                    <p>Edit interactive role assignment messages for {}</p>
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
                                <div style="background: rgba(255, 193, 7, 0.2); border-left: 4px solid #ffc107; padding: 12px; border-radius: 4px; margin-bottom: 15px; color: #fff3cd;">
                                    <strong>ℹ️ Role Hierarchy Notice:</strong> Only roles that are below at least one of the bot's roles in the server hierarchy can be assigned through self-roles. If you don't see a role here, make sure at least one of the bot's roles is positioned above it in Server Settings → Roles.
                                </div>
                                <div class="roles-section">
                                    <div id="rolesList">
                                        <div class="loading">Loading server roles...</div>
                                    </div>
                                </div>
                            </div>
                            
                            <button type="submit" class="btn" id="deployBtn" disabled>
                                Update Self-Role Message
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
                const configId = '{}';
                let channels = [];
                let roles = [];
                
                // Load channels and roles on page load
                document.addEventListener('DOMContentLoaded', async function() {{
                    await Promise.all([loadChannels(), loadRoles()]);
                    await loadExistingConfig();
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
                        
                        if (!data.success) {{
                            // Handle permission or other errors
                            const rolesList = document.getElementById('rolesList');
                            rolesList.innerHTML = `
                                <div style="color: #ffcccb; padding: 15px; background: rgba(220, 53, 69, 0.2); border-radius: 8px; margin-bottom: 15px;">
                                    <strong>⚠️ Error Loading Roles:</strong><br>
                                    ${{data.message || 'Failed to load server roles.'}}
                                    <br><br>
                                    <strong>Common solutions:</strong>
                                    <ul style="margin: 10px 0;">
                                        <li>Ensure the bot is properly added to your server</li>
                                        <li>Make sure the bot has "Manage Roles" permission</li>
                                        <li>Check that the bot's role is above the roles you want to manage</li>
                                        <li>Try re-inviting the bot with proper permissions</li>
                                    </ul>
                                </div>
                            `;
                            return;
                        }}
                        
                        roles = data.roles.filter(role => role.name !== '@everyone').sort((a, b) => b.position - a.position);
                        
                        const rolesList = document.getElementById('rolesList');
                        rolesList.innerHTML = '';
                        
                        if (roles.length === 0) {{
                            rolesList.innerHTML = `
                                <div style="color: #fff3cd; padding: 15px; background: rgba(255, 193, 7, 0.2); border-radius: 8px; margin-bottom: 15px;">
                                    <strong>ℹ️ No Manageable Roles Found</strong><br>
                                    The bot cannot manage any roles in this server. This could be because:
                                    <ul style="margin: 10px 0;">
                                        <li>All server roles are positioned above the bot's highest role</li>
                                        <li>The bot lacks "Manage Roles" permission</li>
                                        <li>No roles have been created in this server yet</li>
                                    </ul>
                                    Please adjust role positions in Server Settings → Roles, or create some roles for the bot to manage.
                                </div>
                            `;
                            return;
                        }}
                        
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
                        document.getElementById('rolesList').innerHTML = `
                            <div style="color: #ffcccb; padding: 15px; background: rgba(220, 53, 69, 0.2); border-radius: 8px;">
                                <strong>❌ Network Error</strong><br>
                                Failed to load roles due to a network error. Please check your connection and try again.
                            </div>
                        `;
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
                
                async function loadExistingConfig() {{
                    try {{
                        const response = await fetch(`/api/selfroles/${{guildId}}/${{configId}}`);
                        const data = await response.json();
                        
                        if (data.success) {{
                            const config = data.config;
                            
                            // Fill in basic fields
                            document.getElementById('title').value = config.title;
                            document.getElementById('body').value = config.body;
                            document.getElementById('channel').value = config.channel_id;
                            
                            // Set selection type
                            document.querySelector(`input[name="selection_type"][value="${{config.selection_type}}"]`).checked = true;
                            
                            // Mark roles as selected and set their emojis
                            config.roles.forEach(configRole => {{
                                const checkbox = document.querySelector(`input[data-role-id="${{configRole.role_id}}"]`);
                                if (checkbox) {{
                                    checkbox.checked = true;
                                    const emojiInput = checkbox.parentElement.querySelector('.emoji-input');
                                    emojiInput.value = configRole.emoji;
                                }}
                            }});
                            
                            // Update the preview and deploy button
                            updatePreview();
                        }}
                    }} catch (error) {{
                        console.error('Failed to load existing config:', error);
                    }}
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
                    deployBtn.textContent = 'Updating...';
                    
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
                        const response = await fetch(`/api/selfroles/${{guildId}}/${{configId}}`, {{
                            method: 'PUT',
                            headers: {{
                                'Content-Type': 'application/json',
                            }},
                            body: JSON.stringify(payload)
                        }});
                        
                        const result = await response.json();
                        
                        if (response.ok && result.success) {{
                            alert('Self-role message updated successfully!');
                            window.location.href = `/dashboard/${{guildId}}/selfroles`;
                        }} else {{
                            alert('Failed to update self-role message: ' + (result.message || 'Unknown error'));
                        }}
                    }} catch (error) {{
                        console.error('Update failed:', error);
                        alert('Failed to update self-role message. Please try again.');
                    }} finally {{
                        deployBtn.disabled = false;
                        deployBtn.textContent = 'Update Self-Role Message';
                    }}
                }});
            </script>
        </body>
        </html>
        "#,
        guild.name,
        guild_id,
        guild.name,
        guild_id,
        config_id
    );
    
    Ok(Html(html))
}
