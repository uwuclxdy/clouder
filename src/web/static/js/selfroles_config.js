// Configuration variables for selfroles form
// This script is dynamically populated based on the form mode (create/edit)

class SelfRoleConfig {
    constructor() {
        // Extract guild ID from URL pattern /dashboard/{guildId}/selfroles/...
        const urlParts = window.location.pathname.split('/');
        this.guildId = urlParts[2]; // /dashboard/{guildId}/selfroles/...
        
        // Check if we're in edit mode by looking for config ID in URL
        // URL patterns:
        // Create: /dashboard/{guild_id}/selfroles/new
        // Edit: /dashboard/{guild_id}/selfroles/edit/{config_id}
        this.configId = null;
        if (urlParts.length > 4 && urlParts[4] === 'edit' && urlParts[5]) {
            this.configId = urlParts[5];
        }
        
        this.isEditMode = this.configId !== null;
    }
    
    getGuildId() {
        return this.guildId;
    }
    
    getConfigId() {
        return this.configId;
    }
    
    isEdit() {
        return this.isEditMode;
    }
}

// Global instance
window.selfRoleConfig = new SelfRoleConfig();