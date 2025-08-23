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
- **DB:** SQLite with shared Arc<SqlitePool> in-app state
- **Auth:** Discord OAuth2 for web interface
- **External APIs:** HuggingFace, GitHub Trending, System stats, Giphy API (for reminders)
- **Background Tasks:** Tokio scheduler for reminders system

## **Bot Functionality**

### **Slash Commands (Server Only)**

#### **Core Commands**
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

**`/purge`** - purges messages from channel (Manage Messages permission)
- signature: `/purge [number / message_id]`
- if number is provided, deletes that many messages
- if message_id is provided, deletes all messages up to that one
- only deletes messages from the channel the command is invoked in
- ephemeral response with number of messages deleted

#### **Reminders**

**`/reminders`** - View active reminders
- **In Server:** Shows all server reminders with their target channels and schedules
- **In DMs:** Shows user's subscribed reminders and personal timezone
- Ephemeral response with formatted list
- Shows reminder type, schedule, and destination

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

### **Welcome/Goodbye Messages System**

#### **Welcome Messages**
- **Automatic Triggers:** Sent when new users join the server (GUILD_MEMBER_ADD event)
- **Channel Configuration:** Administrator selects destination channel
- **Message Types:** Rich embeds or plain text messages
- **Placeholder Support:** Dynamic content with user/server information
- **Independent Control:** Can be enabled/disabled separately from goodbye messages

#### **Goodbye Messages**
- **Automatic Triggers:** Sent when users leave the server (GUILD_MEMBER_REMOVE event)
- **Channel Configuration:** Administrator selects destination channel (can be different from welcome)
- **Message Types:** Rich embeds or plain text messages
- **Placeholder Support:** Dynamic content with user/server information
- **Independent Control:** Can be enabled/disabled separately from welcome messages

#### **Placeholder Variables**
- **`{user}`** - User mention (@username) for pinging
- **`{username}`** - Plain username without mention formatting
- **`{server}`** - Current server/guild name
- **`{member_count}`** - Current total member count
- **`{user_id}`** - User's Discord snowflake ID
- **`{join_date}`** - Account creation date (welcome messages only)

#### **Message Configuration Options**
- **Message Type Selection:**
  - **Text Messages:** Simple plain text with placeholder variable support
  - **Rich Embeds:** Full embed customization with all Discord embed features
- **Embed Configuration (when embed type selected):**
  - Custom title and description with placeholder support
  - Color customization (defaults to server's configured embed color)
  - Footer text with optional timestamp
  - Thumbnail and image URL support
  - Author field configuration
- **Channel Permissions:** Bot validates send message permissions before enabling
- **Test Functionality:** "Send Test Now" buttons for both welcome and goodbye messages

### **Reminders System**

#### **Reminder Types**

**WYSI (When You See It)**
- Triggers at 7:27 AM and 7:27 PM in configured timezone
- Configurable message/embed format
- Supports role pinging
- Shows countdown to next 7:27: `<t:timestamp:R>`

**Femboy Friday**
- Triggers at midnight when timezone enters Friday
- Sends "Happy Femboy Friday :3" with random GIF and mentions all configured roles
- GIF sourced from Giphy API (tagged "femboy friday")
- Configurable message/embed format
- Supports role pinging

**Custom Reminders**
- //TODO: Future implementation
- Placeholder in dashboard (unclickable card)
- Will support cron-like scheduling

#### **Reminder Features**
- **Server Subscriptions:** Administrators can configure reminders for server channels
- **Personal Subscriptions:** Users subscribe to reminders via web dashboard for DM notifications
- **Timezone Support:** Server and user-specific timezones
- **Role Pinging:** Configurable list of roles to ping per reminder (server only)
- **Message Formats:** Rich embeds or plain text messages
- **Test Function:** "Send Test Now" button for administrators
- **Logging:** Execution history and error tracking

### **User facing text**
- **In Discord:** mostly lowercase (titles and abbreviations like OS: capitalized), always mention users instead of saying their username, use short versions of words (Information → Info) short and a little bit silly at times :3
- **Web Dashboard:** Properly capitalized, still short and concise
- **Code comments:** Very concise, no unnecessary words, comments only where logic is not obvious

### **Event Handling**
- **Message Interception:** For uwufy functionality using MESSAGE_CONTENT intent
- **Member Events:** GUILD_MEMBER_ADD and GUILD_MEMBER_REMOVE for welcome/goodbye messages
- **Button Interactions:** For pagination in API commands and self-role assignments
- **Component Interactions:** Handle button clicks for trending periods and role toggles
- **Background Scheduler:** Runs every minute to check and execute due reminders

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
- **Administrator permission required** for server configuration (write access)
- Members without Administrator permission get read-only view
- Only shows mutual servers (where both user and bot are members in)
- Session management with secure cookies

#### **Navigation**
- **Top Navigation Bar:**
  - Logo/Bot name ("clouder")
  - "Add to Server" button (Discord OAuth2 invite with Administrator permission)
  - "User Settings" link
  - User settings button
  - Logout button

#### **Main page:** (`/dashboard`)
- Server list (mutual only)

#### **User Settings Page** (`/settings`)
- **Personal Timezone Configuration:**
  - Dropdown with all timezones
  - Affects DM reminder times
  - Saved to user_settings table
- **DM Reminders Toggle:**
  - Global enable/disable for all DM reminders
- **Subscribed Reminders List:**
  - Shows all reminders user is subscribed to
  - Unsubscribe buttons for each
  - Shows source server for each reminder

#### **Welcome/Goodbye Configuration** (`/dashboard/{guild_id}/welcome-goodbye`)
- **Welcome Message Configuration:**
  - Enable/disable toggle with visual feedback
  - Channel selector (dropdown of available channels)
  - Message type toggle (embed/text) with live UI updates
  - Message content editor with syntax highlighting
  - Placeholder variable reference panel with copy-to-clipboard
  - Live preview pane showing rendered message
  - Embed builder (when embed type selected):
    - Title, description, color picker
    - Footer text with timestamp toggle
    - Thumbnail and image URL inputs with validation
    - Author field configuration
  - "Send Test Welcome" button for immediate testing
- **Goodbye Message Configuration:**
  - Same interface as welcome messages
  - Independent channel selection
  - Independent message type and content
  - "Send Test Goodbye" button for immediate testing
- **Placeholder Variables Reference:**
  - Expandable reference card listing all available variables
  - Click-to-copy functionality for each variable
  - Live examples showing what each placeholder would resolve to
  - Context-sensitive help (e.g., showing current member count)
- **Configuration Status:**
  - Visual indicators for enabled/disabled states
  - Channel permission validation with error messages
  - Bot permission warnings if insufficient permissions

#### **Reminders Management** (`/dashboard/{guild_id}/reminders`)
- **WYSI Configuration:**
  - Enable/disable toggle
  - Channel selector
  - Morning time (default 07:27)
  - Evening time (default 19:27)
  - Timezone setting
  - Role selector for pings
  - Message type (embed/text)
  - Message/embed customization
  - "Test Now" button
- **Femboy Friday Configuration:**
  - Enable/disable toggle
  - Channel selector
  - Trigger time (default 00:00/midnight)
  - Timezone setting
  - Role selector for pings
  - Message type (embed/text)
  - Message/embed customization (GIF auto-included)
  - "Test Now" button
- **Custom Reminders:**
  - Placeholder card marked "Coming Soon"
  - Grayed out/unclickable
  - Shows "//TODO" in description

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
- Timezone settings
- uwufy toggle states
- Command prefix for custom commands
- Embed color settings
- Custom command management per server
- Self-roles configurations and message tracking
- Reminders configurations
- Welcome/goodbye message configurations and status

## **Project Structure**
```
migrations/ # database migrations
├── 001_initial.sql # self-roles
├── 002_reminders.sql # reminders and user settings
├── 003_welcome_goodbye.sql # welcome/goodbye messages
└── ...
src/
├── main.rs
├── .env
├── config.rs # everything configurable here - not spread out across mod files or anywhere else
├── commands/ # all slash commands, organized in folders and files
│   ├── custom/ # code for handling custom commands with configurable per-server prefix
│   └── ...
├── database/ # db related code
│   ├── reminders.rs # reminder-related database operations
│   ├── user_settings.rs # user settings operations
│   ├── welcome_goodbye.rs # welcome/goodbye message operations
│   └── ...
├── events/ # event handlers
│   ├── member_events.rs # welcome/goodbye message handling
│   └── ...
├── scheduler/ # background task scheduler
│   ├── mod.rs
│   └── reminders.rs # reminder execution logic
├── tests/ # all tests here and nowhere else
│   └── ...
├── utils/ # common methods used in multiple commands or other parts of code
│   ├── welcome_goodbye.rs # placeholder replacement and message formatting
│   └── ...
├── web/ # web dashboard related code (use include_str!() macro for html and other static files in order to compile them into binary as well)
│   ├── static/ # js and css here and NOT in .rs files
│   ├── templates/ # html templates here
│   │   ├── reminders_config.html
│   │   ├── user_settings.html
│   │   ├── welcome_goodbye_config.html
│   │   └── ...
│   ├── reminders.rs # reminders web handlers
│   ├── user_settings.rs # user settings web handlers
│   ├── welcome_goodbye.rs # welcome/goodbye web handlers
│   └── ...
└── external/ # external API interactions
    ├── giphy.rs # Giphy API integration (for femboy friday GIFs)
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
- Fail silently and log the error
- Ephemeral error messages to users for interaction failures
- Handle blocked DMs gracefully, unsubscribe that user from reminders
- Handle missing channel permissions for welcome/goodbye messages gracefully
- Log member join/leave event processing errors
- Handle deleted channels in welcome/goodbye configurations

## **Configuration Management**
- Environment variables for secrets (Discord token, OAuth credentials, Giphy API key - update .env.example)
- Config file for non-sensitive settings
- Per-server settings stored in database
- Per-user settings stored in database
- Self-role configurations persisted across restarts
- Reminder configurations persisted across restarts
- Welcome/goodbye configurations persisted across restarts

## **Embed Colors**
```
use crate::utils::get_default_embed_color;

// In commands with Context
.color(get_default_embed_color(ctx.data()))

// In web handlers with AppState
.colour(get_default_embed_color(&state))
```

## **Security Considerations**
- **Administrator permission required** for server configuration write access
- Read-only access for non-administrator members
- Only show mutual servers in dashboard
- Secure session token storage
- Input validation for custom commands and self-role configurations
- Input validation for reminder configurations
- Input validation for welcome/goodbye message content and placeholder usage
- Rate limiting on external API calls
- Proper Discord permission checking
- Role hierarchy validation for self-roles and reminder pings
- Channel permission validation for welcome/goodbye messages
- Cooldown system to prevent role abuse
- DM permission checking before sending reminders
- Sanitize user input in welcome/goodbye messages to prevent abuse

## **Future Enhancements (//TODO)**
- Custom reminders with cron-like scheduling
- Role-based custom command editing
- Live preview for custom commands
- Advanced analytics dashboard
- More sophisticated uwufy algorithms
- Additional external API integrations
- Self-roles usage statistics and audit logs
- Reminder execution analytics
- Giphy API integration for dynamic GIF selection
- Advanced welcome/goodbye message templates
- Role-based welcome messages (different messages for different roles)
- Welcome message DM option (send to user privately instead of channel)
- Advanced placeholder variables (user avatar URL, account age, server boost status, etc.)
- Welcome/goodbye message statistics and analytics
