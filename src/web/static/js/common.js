// Common JavaScript functions used across multiple pages

// API request helper with error handling
async function apiRequest(url, options = {}) {
    try {
        const response = await fetch(url, {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        });
        
        const data = await response.json();
        
        if (!response.ok && !data.success) {
            throw new Error(data.message || `HTTP ${response.status}`);
        }
        
        return { response, data };
    } catch (error) {
        console.error('API request failed:', error);
        throw error;
    }
}

// Show loading state for buttons
function setButtonLoading(button, loading, originalText) {
    if (loading) {
        button.disabled = true;
        button.dataset.originalText = button.textContent;
        button.textContent = originalText || 'Loading...';
    } else {
        button.disabled = false;
        button.textContent = button.dataset.originalText || originalText;
    }
}

// Show success/error messages
function showMessage(message, type = 'info') {
    // Simple alert for now - could be enhanced with custom toast notifications
    if (type === 'error') {
        alert('Error: ' + message);
    } else if (type === 'success') {
        alert('Success: ' + message);
    } else {
        alert(message);
    }
}

// Debounce function for input events
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}