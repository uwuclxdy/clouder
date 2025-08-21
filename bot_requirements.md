# Discord Bot Requirements

## **Core Architecture**
- **Framework:** Serenity + Poise (not Twilight)
- **Web Server:** Axum (same process as Discord bot)
- **Database:** SQLite
- **Language:** Rust
- **Bot Name:** `clouder`

## **Technical Stack**
- **Discord:** Serenity + Poise for commands and event handling
- **Web:** Axum for web dashboard
- **DB:** SQLite with shared Arc<SqlitePool> in app state
- **Auth:** Discord OAuth2 for web interface
- **External APIs:** HuggingFace, GitHub Trending, System stats

## **Bot Functionality**

### **Slash Commands (Server Only)**

#### **Core Commands**
**`/wysi`** - When You See It 
   - countdown to 7:27
   - Uses configurable timezone (default GMT+2)
   - Shows Discord timestamp format: `<t:1748687640:R>`
   - Command: `/wysi timezone` to configure per-server

**`/random`** - Random number between 100000 and 9999999 with website link
   - Format: `https://nhentai.to/g/[number]`
   - Embed response
   - help desc: "freaky link generator"

**`/uwufy [mention]`** - Toggle uwufy mode (Manage Server permission)
   - Deletes user messages and replaces with uwufied text using their pfp/username
   - Persists across restarts until toggled off
   - Only affects new messages (no attachments)
   - Simple character replacement: r/l→w, R/L→W

**`/selfroles`** - Self-roles management (ephemeral response)
   - Sends ephemeral message with link to bot's website for self-role configuration
   - Only visible to command invoker
   - Directs to web dashboard for setup and management

#### **Info Commands**
**`/about`** - Bot and system information
   - Bot version, uptime, made by uwuclxdy
   - System: RAM, CPU, disk usage, network stats, latency
   
**`/about server`** - Discord server statistics and info
   - All available Discord API server data in embed format

**`/about user [@mention - optional]`** - User statistics and info
   - All available Discord API user data in embed format
   - Defaults to command invoker if no mention

**`/help`** - List of all commands and their descriptions
   - Command prefix if configured in the server
   - Command descriptions are in the code

#### **API Integration Commands (with Pagination)**
**`/hg-latest`** - Latest HuggingFace AI models
   - Source: `https://huggingface.co/api/models?sort=lastModified&limit=10`
   - Shows: model name, author, downloads, likes, last modified, category
   - 5 models per page with next/previous buttons
   - 5-minute caching
   
**`/github [user] [repo - optional]`** - GitHub stats and activity
   - User format: `/github octocat`
   - Repo format: `/github octocat Hello-World`
   - Auto-detect user vs repo based on format

**`/gh-trending`** - GitHub trending repositories
    - Source: `https://github.com/isboyjc/github-trending-api`
    - Time period buttons: Daily, Weekly, Monthly
    - Pagination for results

### **User facing text**
- **In Discord:** mostly lowercase (titles and abbreviations like OS: capitalized), always mention users instead of saying their username, use short versions of words (Information → Info) short and a little bit silly at times :3
- **Web Dashboard:** Properly capitalized, still short and concise
- **Code comments:** Very concise, no unnecessary words, comments only where logic is not obvious

### **Event Handling**
- **Message Interception:** For uwufy functionality using MESSAGE_CONTENT intent
- **Button Interactions:** For pagination in API commands and self-role assignments
- **Component Interactions:** Handle button clicks for trending periods and role toggles

### **Self-Roles System**

#### **Functionality**
- **Multiple Configurations:** Each server can have multiple self-role setups across different channels
- **Selection Types:** 
  - **Radio Mode:** Users can select only one role from the message (removes others)
  - **Multiple Mode:** Users can select multiple roles from the same message
- **Role Management:** Users click buttons to add/remove roles
- **Error Handling:** Ephemeral messages for any errors (permission issues, role hierarchy, etc.)
- **Cooldown:** 5-second cooldown per user per specific role (prevents role spam)
- **Message Tracking:** All self-role messages tracked in database for updates/restarts

#### **Security & Validation**
- **Permissions:** Manage Roles permission required to configure
- **Hierarchy Check:** Bot validates it can assign roles (role hierarchy)
- **Error Messages:** Clear ephemeral feedback for permission/hierarchy issues

### **Web Dashboard Features**

#### **Authentication**
- Discord OAuth2 login
- Server permission validation (Manage Server required for general config, Manage Roles for self-roles)
- Session management with secure cookies

#### **Custom Commands Management**
- **Command Name:** 20 character limit
- **Command Type:** NOT a slash command, configurable command prefix per server
- **Output Types:** 
  - Simple text message
  - Rich embeds with fields
- **//TODO:** Role-based delegation, live preview

#### **Self-Roles Configuration**
- **Server Selection:** Must be logged in and have Manage Roles permission
- **Channel Selection:** Choose target channel for self-role message
- **Message Configuration:**
  - Custom embed title and body (no character limits)
  - Selection type: Radio (single) or Multiple selection
  - Role selection with emoji assignment (Discord default + custom from same server)
  - Live preview of final embed and buttons
- **Role Management:**
  - Visual role selector with emoji picker
  - Automatic hierarchy validation
  - Real-time preview updates
- **Message Updates:** Edit existing self-role messages when configuration changes
- **Multiple Setups:** Create multiple self-role configurations per server

#### **Server Configuration**
- Timezone settings for `/wysi` command
- uwufy toggle states
- Custom command management per server
- Self-roles configurations and message tracking

## **Project Structure**
```
migrations/ # database migrations
└── ...
src/
├── main.rs
├── .env
├── config.rs # everything configurable here - not spread out across mod files or anywhere else
├── commands/ # all slash commands, organized in folders and files
│   ├── custom/ # code for handling custom commands with configurable per-server prefixe
│   └── ...
├── database/ # db related code
│   └── ...
├── events/ # event handlers
│   └── ...
├── tests/ # all tests here and nowhere else
│   └── ...
├── utils/ # common methods used in multiple commands or other parts of code
│   └── ...
├── web/ # web dashboard related code (use include_str!() macro for html and other static files in order to compile them into binary as well)
│   ├── static/ # js and css here and NOT in .rs files
│   ├── templates/ # html templates here
│   └── ...
└── external/ # external API interactions
    └── ...
```

## **Structure in production**
- Auto-creates `data` folder, db file (if not exists) and tables on startup with `IF NOT EXISTS`
```
├── clouder.exe
└── data/
    └── db.sqlite
```

### **Error Handling**
- fail silently and log the error
- ephemeral error messages to users for interaction failures

### **External Dependencies (Estimated)**
- do not worry about dependencies or creating the cargo.toml file, i can easily autoimport missing references after pasting the code. make sure that all of the packages you use are valid with web search and also check on their docs for correct usage with web search tool.

## **Configuration Management**
- Environment variables for secrets (Discord token, OAuth credentials)
- Config file for non-sensitive settings
- Per-server settings stored in database
- Self-role configurations persisted across restarts

## **Embed Colors**
```
use crate::utils::get_default_embed_color;

// In commands with Context
.color(get_default_embed_color(ctx.data()))

// In web handlers with AppState
.colour(get_default_embed_color(&state))
```

## **Security Considerations**
- Secure session token storage
- Input validation for custom commands and self-role configurations
- Rate limiting on external API calls
- Proper Discord permission checking
- Role hierarchy validation for self-roles
- Cooldown system to prevent role abuse

## **Future Enhancements (//TODO)**
- Role-based custom command editing
- Live preview for custom commands
- Advanced analytics dashboard
- More sophisticated uwufy algorithms
- Additional external API integrations
- Self-roles usage statistics and audit logs
