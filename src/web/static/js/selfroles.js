// Self-roles management JavaScript

class SelfRoleManager {
    constructor(guildId, configId = null) {
        this.guildId = guildId;
        this.configId = configId;
        this.channels = [];
        this.roles = [];
        this.isEditMode = configId !== null;
        
        this.initializeEventListeners();
    }

    initializeEventListeners() {
        // Form input listeners
        const titleInput = document.getElementById('title');
        const bodyInput = document.getElementById('body');
        
        if (titleInput) titleInput.addEventListener('input', debounce(() => this.updatePreview(), 300));
        if (bodyInput) bodyInput.addEventListener('input', debounce(() => this.updatePreview(), 300));
        
        const channelSelect = document.getElementById('channel');
        if (channelSelect) channelSelect.addEventListener('change', () => this.updateDeployButton());
        
        // Selection type radio buttons
        const selectionTypeRadios = document.querySelectorAll('input[name="selection_type"]');
        selectionTypeRadios.forEach(radio => {
            radio.addEventListener('change', () => this.updatePreview());
        });

        // Form submission
        const form = document.getElementById('selfRoleForm');
        if (form) {
            form.addEventListener('submit', (e) => this.handleFormSubmit(e));
        }
    }

    async init() {
        try {
            await Promise.all([this.loadChannels(), this.loadRoles()]);
            
            if (this.isEditMode) {
                await this.loadExistingConfig();
            }
            
            this.updateDeployButton();
        } catch (error) {
            console.error('Failed to initialize:', error);
            showMessage('Failed to load initial data. Please refresh the page.', 'error');
        }
    }

    async loadChannels() {
        try {
            const { data } = await apiRequest(`/api/guild/${this.guildId}/channels`);
            this.channels = data.channels.filter(ch => ch.type === 0); // Text channels only

            const channelSelect = document.getElementById('channel');
            channelSelect.innerHTML = '<option value="">Select a channel...</option>';
            
            this.channels.forEach(channel => {
                channelSelect.innerHTML += `<option value="${channel.id}">#${channel.name}</option>`;
            });
        } catch (error) {
            console.error('Failed to load channels:', error);
            document.getElementById('channel').innerHTML = '<option value="">Failed to load channels</option>';
        }
    }

    async loadRoles() {
        try {
            const { data } = await apiRequest(`/api/guild/${this.guildId}/roles`);

            if (!data.success) {
                this.displayRoleLoadError(data.message || 'Failed to load server roles.');
                return;
            }

            this.roles = data.roles.filter(role => role.name !== '@everyone')
                                   .sort((a, b) => b.position - a.position);

            const rolesList = document.getElementById('rolesList');
            rolesList.innerHTML = '';

            if (this.roles.length === 0) {
                this.displayNoManageableRoles();
                return;
            }

            this.roles.forEach(role => {
                const roleItem = this.createRoleItem(role);
                rolesList.appendChild(roleItem);
            });
        } catch (error) {
            console.error('Failed to load roles:', error);
            this.displayNetworkError();
        }
    }

    displayRoleLoadError(message) {
        const rolesList = document.getElementById('rolesList');
        rolesList.innerHTML = `
            <div style="color: #ffcccb; padding: 15px; background: rgba(220, 53, 69, 0.2); border-radius: 8px; margin-bottom: 15px;">
                <strong>⚠️ Error Loading Roles:</strong><br>
                ${message}
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
    }

    displayNoManageableRoles() {
        const rolesList = document.getElementById('rolesList');
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
    }

    displayNetworkError() {
        const rolesList = document.getElementById('rolesList');
        rolesList.innerHTML = `
            <div style="color: #ffcccb; padding: 15px; background: rgba(220, 53, 69, 0.2); border-radius: 8px;">
                <strong>❌ Network Error</strong><br>
                Failed to load roles due to a network error. Please check your connection and try again.
            </div>
        `;
    }

    createRoleItem(role) {
        const roleItem = document.createElement('div');
        roleItem.className = 'role-item';
        roleItem.innerHTML = `
            <input type="checkbox" class="role-checkbox" data-role-id="${role.id}" onchange="selfRoleManager.updatePreview()">
            <input type="text" class="emoji-input" placeholder="🎮" maxlength="2" onchange="selfRoleManager.updatePreview()">
            <div class="role-name" style="color: #${role.color.toString(16).padStart(6, '0')}">${role.name}</div>
        `;
        return roleItem;
    }

    updatePreview() {
        const title = document.getElementById('title').value || 'Choose your roles';
        const body = document.getElementById('body').value || 'Click the buttons below to assign yourself roles...';

        document.getElementById('previewTitle').textContent = title;
        document.getElementById('previewBody').textContent = body;

        const previewButtons = document.getElementById('previewButtons');
        previewButtons.innerHTML = '';

        const checkboxes = document.querySelectorAll('.role-checkbox:checked');
        checkboxes.forEach(checkbox => {
            const roleId = checkbox.dataset.roleId;
            const role = this.roles.find(r => r.id === roleId);
            const emojiInput = checkbox.parentElement.querySelector('.emoji-input');
            const emoji = emojiInput.value || '📝';

            if (role) {
                const button = document.createElement('button');
                button.className = 'preview-button';
                button.innerHTML = `${emoji} ${role.name}`;
                previewButtons.appendChild(button);
            }
        });

        this.updateDeployButton();
    }

    updateDeployButton() {
        const channel = document.getElementById('channel').value;
        const title = document.getElementById('title').value;
        const body = document.getElementById('body').value;
        const selectedRoles = document.querySelectorAll('.role-checkbox:checked');

        const deployBtn = document.getElementById('deployBtn');
        deployBtn.disabled = !channel || !title || !body || selectedRoles.length === 0;
    }

    async loadExistingConfig() {
        try {
            const { data } = await apiRequest(`/api/selfroles/${this.guildId}/${this.configId}`);

            if (data.success) {
                const config = data.config;

                // Fill in basic fields
                document.getElementById('title').value = config.title;
                document.getElementById('body').value = config.body;
                document.getElementById('channel').value = config.channel_id;

                // Set selection type
                const selectionTypeRadio = document.querySelector(`input[name="selection_type"][value="${config.selection_type}"]`);
                if (selectionTypeRadio) selectionTypeRadio.checked = true;

                // Mark roles as selected and set their emojis
                config.roles.forEach(configRole => {
                    const checkbox = document.querySelector(`input[data-role-id="${configRole.role_id}"]`);
                    if (checkbox) {
                        checkbox.checked = true;
                        const emojiInput = checkbox.parentElement.querySelector('.emoji-input');
                        emojiInput.value = configRole.emoji;
                    }
                });

                // Update the preview and deploy button
                this.updatePreview();
            }
        } catch (error) {
            console.error('Failed to load existing config:', error);
            showMessage('Failed to load existing configuration.', 'error');
        }
    }

    async handleFormSubmit(event) {
        event.preventDefault();

        const deployBtn = document.getElementById('deployBtn');
        const originalText = deployBtn.textContent;
        const actionText = this.isEditMode ? 'Updating...' : 'Deploying...';
        
        setButtonLoading(deployBtn, true, actionText);

        try {
            const formData = new FormData(event.target);
            const selectedRoles = [];

            document.querySelectorAll('.role-checkbox:checked').forEach(checkbox => {
                const roleId = checkbox.dataset.roleId;
                const emoji = checkbox.parentElement.querySelector('.emoji-input').value || '📝';
                selectedRoles.push({ role_id: roleId, emoji: emoji });
            });

            const payload = {
                title: formData.get('title'),
                body: formData.get('body'),
                selection_type: formData.get('selection_type'),
                channel_id: formData.get('channel_id'),
                roles: selectedRoles
            };

            const url = this.isEditMode 
                ? `/api/selfroles/${this.guildId}/${this.configId}`
                : `/api/selfroles/${this.guildId}`;
            
            const method = this.isEditMode ? 'PUT' : 'POST';

            const { data } = await apiRequest(url, { method, body: JSON.stringify(payload) });

            if (data.success) {
                const successMessage = this.isEditMode 
                    ? 'Self-role message updated successfully!'
                    : 'Self-role message deployed successfully!';
                
                showMessage(successMessage, 'success');
                window.location.href = `/dashboard/${this.guildId}/selfroles`;
            } else {
                throw new Error(data.message || 'Unknown error');
            }
        } catch (error) {
            console.error('Operation failed:', error);
            showMessage(error.message || 'Operation failed. Please try again.', 'error');
        } finally {
            setButtonLoading(deployBtn, false, originalText);
        }
    }
}

// Self-roles list page functions
async function loadMessages(guildId) {
    try {
        const { data } = await apiRequest(`/api/selfroles/${guildId}`);
        const messagesGrid = document.getElementById('messagesGrid');

        if (data.success && data.configs.length > 0) {
            messagesGrid.innerHTML = '';

            data.configs.forEach(config => {
                const messageCard = createMessageCard(config, guildId);
                messagesGrid.appendChild(messageCard);
            });
        } else {
            displayNoMessages(messagesGrid, guildId);
        }
    } catch (error) {
        console.error('Failed to load messages:', error);
        displayLoadError(document.getElementById('messagesGrid'));
    }
}

function createMessageCard(config, guildId) {
    const messageCard = document.createElement('div');
    messageCard.className = 'message-card';

    const createdDate = new Date(config.created_at).toLocaleDateString();

    messageCard.innerHTML = `
        <div class="message-title">${config.title}</div>
        <div class="message-body">${config.body}</div>
        <div class="message-meta">
            <span>Created: ${createdDate}</span>
            <div>
                <span class="role-count">${config.role_count} roles</span>
                <span class="selection-type">${config.selection_type}</span>
            </div>
        </div>
        <div class="message-actions">
            <a href="/dashboard/${guildId}/selfroles/edit/${config.id}" class="edit-btn">Edit</a>
            <button class="delete-btn" onclick="deleteMessage(event, ${config.id}, '${config.title.replace(/'/g, "\\'")}', '${guildId}')">Delete</button>
        </div>
    `;

    return messageCard;
}

function displayNoMessages(messagesGrid, guildId) {
    messagesGrid.innerHTML = `
        <div class="no-messages">
            <h3>No self-role messages yet</h3>
            <p>Create your first interactive role assignment message to get started!</p>
            <a href="/dashboard/${guildId}/selfroles/new" class="create-btn">+ Create First Message</a>
        </div>
    `;
}

function displayLoadError(messagesGrid) {
    messagesGrid.innerHTML = `
        <div class="no-messages">
            <h3>Failed to load messages</h3>
            <p>There was an error loading your self-role messages. Please try refreshing the page.</p>
        </div>
    `;
}

async function deleteMessage(event, configId, title, guildId) {
    event.stopPropagation();

    if (!confirm(`Are you sure you want to delete "${title}"? This will also remove the message from Discord and cannot be undone.`)) {
        return;
    }

    try {
        const { data } = await apiRequest(`/api/selfroles/${guildId}/${configId}`, { method: 'DELETE' });

        if (data.success) {
            showMessage('Self-role message deleted successfully!', 'success');
            await loadMessages(guildId); // Reload the messages list
        } else {
            throw new Error(data.message || 'Unknown error');
        }
    } catch (error) {
        console.error('Delete failed:', error);
        showMessage(error.message || 'Failed to delete self-role message. Please try again.', 'error');
    }
}

// Global variable for the current instance
let selfRoleManager = null;