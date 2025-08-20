// Self-roles management

class SelfRoleManager {
    constructor(guildId, configId = null) {
        this.guildId = guildId;
        this.configId = configId;
        this.channels = [];
        this.roles = [];
        this.isEditMode = configId !== null;
        this.currentEmojiRoleId = null;
        
        this.initializeEventListeners();
    }

    initializeEventListeners() {
        const titleInput = document.getElementById('title');
        const bodyInput = document.getElementById('body');
        
        if (titleInput) titleInput.addEventListener('input', debounce(() => this.updatePreview(), 300));
        if (bodyInput) bodyInput.addEventListener('input', debounce(() => this.updatePreview(), 300));
        
        const channelSelect = document.getElementById('channel');
        if (channelSelect) channelSelect.addEventListener('change', () => this.updateDeployButton());
        
        const selectionTypeRadios = document.querySelectorAll('input[name="selection_type"]');
        selectionTypeRadios.forEach(radio => {
            radio.addEventListener('change', () => {
                this.updatePreview();
                this.updateRadioGroupStyles();
            });
        });
        
        this.updateRadioGroupStyles();

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
            this.channels = data.channels.filter(ch => ch.type === 0);

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
                <strong>❌ Role Loading Error</strong><br>
                ${escapeHtml(message)}
                <br><br>
                <button onclick="location.reload()" class="btn" style="margin-top: 10px;">Retry</button>
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
                <br><br>
                <button onclick="location.reload()" class="btn" style="margin-top: 10px;">Retry</button>
            </div>
        `;
    }

    createRoleItem(role) {
        const roleItem = document.createElement('div');
        roleItem.className = 'role-item';
        
        roleItem.innerHTML = `
            <input type="checkbox" class="role-checkbox" data-role-id="${role.id}" onchange="selfRoleManager.updatePreview(); selfRoleManager.updateRoleItemState(this)" onclick="event.stopPropagation()">
            <div class="emoji-button" data-role-id="${role.id}">
                <span class="emoji-text">🎮</span>
            </div>
            <div class="role-name">${role.name}</div>
            <div class="role-color-indicator" style="background-color: #${role.color.toString(16).padStart(6, '0')}"></div>
        `;
        
        // Set up emoji button click handler
        const emojiButton = roleItem.querySelector('.emoji-button');
        emojiButton.addEventListener('click', (e) => {
            e.stopPropagation();
            this.openEmojiPicker(role.id);
        });
        
        // Clickable role item
        roleItem.addEventListener('click', (e) => {
            if (!e.target.closest('.emoji-button')) {
                this.addRippleEffect(roleItem);
                
                const checkbox = roleItem.querySelector('.role-checkbox');
                checkbox.checked = !checkbox.checked;
                this.updateRoleItemState(checkbox);
                this.updatePreview();
            }
        });
        
        return roleItem;
    }

    updateRoleItemState(checkbox) {
        const roleItem = checkbox.closest('.role-item');
        if (checkbox.checked) {
            roleItem.classList.add('selected');
        } else {
            roleItem.classList.remove('selected');
        }
    }

    addRippleEffect(element) {
        element.classList.add('ripple');
        setTimeout(() => {
            element.classList.remove('ripple');
        }, 300);
    }

    openEmojiPicker(roleId) {
        this.currentEmojiRoleId = roleId;
        
        // Create modal if it doesn't exist
        if (!document.getElementById('emojiPickerModal')) {
            this.createEmojiPickerModal();
        }
        
        const modal = document.getElementById('emojiPickerModal');
        modal.classList.add('show');
        
        // Close on backdrop click
        modal.addEventListener('click', (e) => {
            if (e.target === modal) {
                this.closeEmojiPicker();
            }
        });
        
        // Close on escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                this.closeEmojiPicker();
            }
        }, { once: true });
    }

    closeEmojiPicker() {
        const modal = document.getElementById('emojiPickerModal');
        if (modal) {
            modal.classList.remove('show');
        }
    }

    createEmojiPickerModal() {
        const modal = document.createElement('div');
        modal.id = 'emojiPickerModal';
        modal.innerHTML = `
            <div id="emojiPickerContent">
                <emoji-picker></emoji-picker>
            </div>
        `;
        
        document.body.appendChild(modal);
        
        // Set up emoji selection
        setTimeout(() => {
            const picker = modal.querySelector('emoji-picker');
            if (picker) {
                picker.addEventListener('emoji-click', (event) => {
                    const emoji = event.detail.unicode;
                    this.setRoleEmoji(this.currentEmojiRoleId, emoji);
                    this.closeEmojiPicker();
                });
            }
        }, 100);
    }

    setRoleEmoji(roleId, emoji) {
        const emojiButton = document.querySelector(`[data-role-id="${roleId}"].emoji-button`);
        if (emojiButton) {
            const emojiText = emojiButton.querySelector('.emoji-text');
            emojiText.textContent = emoji;
            emojiButton.classList.add('has-emoji');
            this.updatePreview();
        }
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
            const emojiButton = checkbox.parentElement.querySelector('.emoji-button .emoji-text');
            const emoji = emojiButton ? emojiButton.textContent : '📝';

            if (role) {
                const button = document.createElement('button');
                button.className = 'preview-button';
                button.innerHTML = `${emoji} ${role.name}`;
                previewButtons.appendChild(button);
            }
        });

        this.updateDeployButton();
    }

    updateRadioGroupStyles() {
        const radioGroups = document.querySelectorAll('.radio-group');
        radioGroups.forEach(group => {
            const radio = group.querySelector('input[type="radio"]');
            if (radio.checked) {
                group.classList.add('selected');
            } else {
                group.classList.remove('selected');
            }
        });
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

                // Set selected roles and emojis
                config.roles.forEach(configRole => {
                    const checkbox = document.querySelector(`input[data-role-id="${configRole.role_id}"]`);
                    if (checkbox) {
                        checkbox.checked = true;
                        this.updateRoleItemState(checkbox);
                        this.setRoleEmoji(configRole.role_id, configRole.emoji);
                    }
                });

                // Update preview
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
                const emojiButton = checkbox.parentElement.querySelector('.emoji-button .emoji-text');
                const emoji = emojiButton ? emojiButton.textContent : '📝';
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

            data.configs.forEach((config, index) => {
                const messageCard = createMessageCard(config, guildId);
                // Add staggered animation delay
                messageCard.style.animationDelay = `${index * 0.1}s`;
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
    const channelName = config.channel_name || 'Unknown Channel';

    // Click to edit
    messageCard.addEventListener('click', (e) => {
        // Skip delete button clicks
        if (!e.target.classList.contains('delete-btn')) {
            window.location.href = `/dashboard/${guildId}/selfroles/edit/${config.id}`;
        }
    });

    messageCard.innerHTML = `
        <div class="message-badges">
            <span class="role-count">${config.role_count} roles</span>
            <span class="selection-type">${config.selection_type}</span>
        </div>
        <div class="message-card-content">
            <div class="message-title">${config.title}</div>
            <div class="message-body">${config.body}</div>
            <div class="message-info">
                <div class="message-channel">${channelName}</div>
                <div class="message-meta">
                    <div class="message-meta-left">
                        <span>Created: ${createdDate}</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="message-actions">
            <button class="delete-btn" onclick="deleteMessage(event, ${config.id}, '${config.title.replace(/'/g, "\\'")}', '${guildId}')">Delete</button>
        </div>
    `;

    return messageCard;
}

function displayNoMessages(messagesGrid, guildId) {
    messagesGrid.innerHTML = `
        <div class="no-messages">
            <div style="font-size: 4em; margin-bottom: 20px; opacity: 0.6;">📝</div>
            <h3>No self-role messages yet</h3>
            <p>Create your first interactive role assignment message to get started!</p>
            <a href="/dashboard/${guildId}/selfroles/new" class="create-btn">+ Create First Message</a>
        </div>
    `;
}

function displayLoadError(messagesGrid) {
    messagesGrid.innerHTML = `
        <div class="no-messages">
            <div style="font-size: 4em; margin-bottom: 20px; opacity: 0.6;">⚠️</div>
            <h3>Failed to load messages</h3>
            <p>There was an error loading your self-role messages. Please try refreshing the page.</p>
            <button class="btn" onclick="location.reload()">Refresh Page</button>
        </div>
    `;
}

async function deleteMessage(event, configId, title, guildId) {
    event.stopPropagation();

    const confirmed = confirm(`⚠️ Delete "${title}"?\n\nThis will permanently remove:\n• The self-role message from Discord\n• All role assignment data\n• Cannot be undone\n\nAre you sure?`);
    
    if (!confirmed) {
        return;
    }

    const deleteBtn = event.target;
    const originalText = deleteBtn.textContent;
    deleteBtn.textContent = 'Deleting...';
    deleteBtn.disabled = true;

    try {
        const { data } = await apiRequest(`/api/selfroles/${guildId}/${configId}`, { method: 'DELETE' });

        if (data.success) {
            showMessage('Self-role message deleted successfully!', 'success');
            await loadMessages(guildId); // Reload messages
        } else {
            throw new Error(data.message || 'Unknown error');
        }
    } catch (error) {
        console.error('Delete failed:', error);
        showMessage(error.message || 'Failed to delete self-role message. Please try again.', 'error');
        
        // Reset button state on error
        deleteBtn.textContent = originalText;
        deleteBtn.disabled = false;
    }
}

// Manager instance
let selfRoleManager = null;