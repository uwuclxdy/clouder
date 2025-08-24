# Clouder Discord Bot - Detailed Implementation Checklist

> subtasks of completed main tasks are redacted

---

## **Database Schema & Models**
> **Note:** create only the tables needed for the functionality you are currently working on!

### Current Tables
- [x] `selfrole_configs` table - stores self-role configuration metadata
- [x] `selfrole_roles` table - stores role-emoji mappings for self-roles
- [x] `selfrole_cooldowns` table - prevents role spam with cooldowns

### Reminders System Tables [x]
- [x] done subtasks redacted

### Welcome/Goodbye System Tables [x]
- [x] done subtasks redacted

### Additional Tables (Future) [ ]
- [ ] `uwufy_toggles` table
  - [ ] Add `guild_id` (TEXT)
  - [ ] Add `user_id` (TEXT)
  - [ ] Add `enabled` (BOOLEAN)
  - [ ] Add `toggled_at` timestamp
  - [ ] Add PRIMARY KEY(guild_id, user_id)
- [ ] `custom_commands` table
  - [ ] Add all necessary fields for custom commands
  - [ ] Add permission and ownership tracking
- [ ] `api_cache` table
  - [ ] Add caching for external API calls

### Database Model Implementations (`src/database/*`) [x]
- [x] done subtasks redacted

---

## **Event Handling System**

### Message Interception for Uwufy [ ]
- [ ] Create `src/events/message_handler.rs`
- [ ] Implement MESSAGE_CONTENT intent handling
- [ ] Message processing pipeline
- [ ] Webhook creation for uwufied messages
- [ ] Handle permission errors

### Member Events for Welcome/Goodbye [x]
- [x] done subtasks redacted

### Button Interactions [ ]
- [x] Self-role button interactions (already implemented)
- [ ] Test reminder button handlers
- [ ] Subscription button handlers
- [ ] Welcome/goodbye test message handlers
- [ ] HuggingFace pagination buttons
- [ ] GitHub trending time period buttons
- [ ] GitHub trending pagination buttons

### Component Interactions [ ]
- [ ] Create unified component handler
- [ ] Route based on custom_id prefix
- [ ] Handle expired interactions

---

## **Scheduler System** [x]

### Background Task Infrastructure (`src/scheduler/*`) [x]
- [x] done subtasks redacted

### Reminder Execution Logic [ ]
- [ ] WYSI reminder execution
  - [ ] Calculate next 7:27 AM/PM in timezone
  - [ ] Format countdown message
  - [ ] Send to configured channel with role pings
  - [ ] Send DMs to subscribers in their timezone
- [ ] Femboy Friday execution
  - [ ] Detect Friday midnight in timezone
  - [ ] Fetch random GIF from Giphy API (placeholder for now)
  - [ ] Send message with GIF
  - [ ] Handle role pings
  - [ ] Send DMs to subscribers
- [ ] Test reminder functionality
  - [ ] Implement immediate execution for "Test Now" button
  - [ ] Skip scheduling, execute directly

---

## **Slash Commands Implementation**

### Core Commands

#### `/reminders` - View active reminders [x]
- [x] done subtasks redacted

#### `/random` - Random Number Generator [ ]
- [ ] Create `src/commands/random.rs`
- [ ] Implement main command function
  - [ ] Generate random number between 100000-9999999
  - [ ] Create embed with number and link: `https://nhentai.to/g/[number]`
  - [ ] Style embed with appropriate color and formatting
  - [ ] Send public response
- [ ] Register command in `src/main.rs`

#### `/uwufy [mention]` - Toggle Uwufy Mode [ ]
- [ ] Create `src/commands/uwufy.rs`
- [ ] Implement command function
  - [ ] Check user has Manage Server permission
  - [ ] Parse mentioned user from command
  - [ ] Toggle uwufy state in database
  - [ ] Send confirmation message
- [ ] Create uwufy utility functions in `src/utils/uwufy.rs`
  - [ ] `uwufy_text()` function (r/l→w, R/L→W replacements)
  - [ ] `is_uwufy_enabled()` function
- [ ] Register command in `src/main.rs`

#### `/selfroles` [x]
- [x] done subtasks redacted

#### `/purge` [x]
- [x] done subtasks redacted

### Info Commands

#### `/about` [x]
- [x] done subtasks redacted

#### `/help` [x]
- [x] Lists all commands with descriptions

### API Integration Commands

#### `/hg-latest` - HuggingFace Models [ ]
- [ ] Create `src/commands/huggingface.rs`
- [ ] Create `src/external/huggingface.rs`
  - [ ] API integration
  - [ ] Pagination system
  - [ ] 5-minute caching
- [ ] Register command

#### `/github [user] [repo]` - GitHub Integration [ ]
- [ ] Create `src/commands/github.rs`
- [ ] Create `src/external/github.rs`
  - [ ] User/repo detection logic
  - [ ] API integration
- [ ] Register command

#### `/gh-trending` - GitHub Trending [ ]
- [ ] Create `src/commands/github_trending.rs`
- [ ] Create `src/external/github_trending.rs`
  - [ ] Time period buttons
  - [ ] Pagination
- [ ] Register command

---

## **Web Dashboard Enhancements**

### Navigation Updates [ ]
- [ ] Update top navigation bar
  - [ ] Add "Add to Server" button
    - [ ] Discord OAuth2 invite link
    - [ ] Pre-select Administrator permission
    - [ ] Open in new tab
  - [ ] Add "User Settings" link
  - [ ] Keep existing elements (logo, logout)
- [ ] Update `src/web/templates/partials/navigation.html` (create if needed)

### Permission System Overhaul [ ]
- [ ] Update all permission checks from "Manage Server" to "Administrator"
  - [ ] Update `src/web/middleware.rs`
  - [ ] Update dashboard access logic
  - [ ] Add read-only mode for non-administrators
- [ ] Implement mutual server filtering
  - [ ] Only show servers where both user and bot are members
  - [ ] Update server list query logic
  - [ ] Hide non-mutual servers

### User Settings Page [ ]
- [ ] Create `src/web/templates/user_settings.html`
  - [ ] Timezone selector dropdown
  - [ ] DM reminders enable/disable toggle
  - [ ] List of subscribed reminders
  - [ ] Unsubscribe buttons
- [ ] Create `src/web/user_settings.rs`
  - [ ] GET `/settings` - display user settings
  - [ ] POST `/api/user/timezone` - update timezone
  - [ ] POST `/api/user/dm_reminders` - toggle DM reminders
  - [ ] DELETE `/api/user/subscription/{id}` - unsubscribe from reminder
- [ ] Add user settings link to navigation

### Reminders Configuration Page [ ]
- [ ] Create `src/web/templates/reminders_config.html`
  - [ ] WYSI configuration section
    - [ ] Enable/disable toggle
    - [ ] Channel selector
    - [ ] Morning/evening time pickers
    - [ ] Timezone selector
    - [ ] Role ping selector (multi-select)
    - [ ] Message type toggle (embed/text)
    - [ ] Message content editor
    - [ ] Embed builder (if embed type)
    - [ ] "Test Now" button
  - [ ] Femboy Friday configuration section
    - [ ] Enable/disable toggle
    - [ ] Channel selector
    - [ ] Trigger time picker (midnight default)
    - [ ] Timezone selector
    - [ ] Role ping selector
    - [ ] Message customization
    - [ ] GIF preview (placeholder)
    - [ ] "Test Now" button
  - [ ] Custom reminders placeholder
    - [ ] "Coming Soon" card
    - [ ] Grayed out/disabled
    - [ ] TODO notice
- [ ] Create `src/web/reminders.rs`
  - [ ] GET `/dashboard/{guild_id}/reminders` - display config page
  - [ ] POST `/api/reminders/{guild_id}/wysi` - save WYSI config
  - [ ] POST `/api/reminders/{guild_id}/femboy_friday` - save FF config
  - [ ] POST `/api/reminders/{guild_id}/test` - trigger test
  - [ ] GET `/api/reminders/{guild_id}/subscriptions` - get subscribers
- [ ] Create `src/web/static/js/reminders_config.js`
  - [ ] Role selector functionality
  - [ ] Message type toggle logic
  - [ ] Embed builder UI
  - [ ] Test button handlers
  - [ ] Form validation

### Welcome/Goodbye Configuration Page [x]
- [x] done subtasks redacted

### User Subscription Management [ ]
- [ ] Add subscription UI to reminders page
  - [ ] "Subscribe for DMs" button for each reminder
  - [ ] Show current subscription status
  - [ ] Handle subscribe/unsubscribe actions
- [ ] Implement subscription API endpoints
  - [ ] POST `/api/user/subscribe/{config_id}`
  - [ ] DELETE `/api/user/unsubscribe/{config_id}`
  - [ ] GET `/api/user/subscriptions`

### Server Configuration Updates [ ]
- [ ] Update `src/web/templates/guild_dashboard.html`
  - [ ] Add reminders section link
  - [ ] Add welcome/goodbye section link
  - [ ] Show reminder summary/status
  - [ ] Show welcome/goodbye configuration status
- [ ] Update server config to include
  - [ ] Command prefix setting
  - [ ] Default embed color
  - [ ] Reminder counts/status
  - [ ] Welcome/goodbye status indicators

### Read-Only Mode [ ]
- [ ] Implement read-only view for non-administrators
  - [ ] Show current configuration
  - [ ] Disable all form inputs
  - [ ] Hide save/test buttons
  - [ ] Display permission notice
- [ ] Add permission check on all API endpoints
  - [ ] Return 403 for non-administrators
  - [ ] Except for read endpoints

### Custom Commands Management [ ]
- [ ] Create custom commands UI (future)
- [ ] Implement prefix-based commands
- [ ] Role-based delegation (TODO)

---

## **External API Integration**

### Giphy API [ ]
- [ ] Add `GIPHY_API_KEY` to `.env`
- [ ] Create `src/external/giphy.rs`
  - [ ] `search_gifs()` function
  - [ ] `get_random_gif()` function with tag
  - [ ] Rate limiting
  - [ ] Error handling
- [ ] Integrate with Femboy Friday reminder
  - [ ] Search for "femboy friday" tag
  - [ ] Cache GIF URLs
  - [ ] Fallback handling

### HuggingFace API [ ]
- [ ] Implement API client
- [ ] Add caching logic
- [ ] Error handling

### GitHub API [ ]
- [ ] Add GitHub token to config
- [ ] Implement GraphQL queries
- [ ] Rate limiting

### GitHub Trending API [ ]
- [ ] Implement trending API client
- [ ] Time period handling
- [ ] Data parsing

---

## **Configuration Management**

### Environment Variables [ ]
- [ ] Add to `.env.example` file:
  - [ ] `GIPHY_API_KEY=` for Giphy API
  - [ ] `GITHUB_TOKEN=` for GitHub API
  - [ ] `HUGGINGFACE_TOKEN=` (if needed)
  - [ ] `DEFAULT_TIMEZONE=UTC`
  - [ ] `CACHE_DURATION=300` (5 minutes)
  - [ ] `SCHEDULER_INTERVAL=60` (1 minute)

### Config Updates [ ]
- [ ] Update `src/config.rs`
  - [ ] Add `giphy_api_key` field
  - [ ] Add `github_token` field
  - [ ] Add `default_timezone` field
  - [ ] Add `scheduler_interval` field
  - [ ] Add reminder-specific configs

---

## **Utility Functions**

### Text Processing [ ]
- [ ] Create `src/utils/uwufy.rs`
  - [ ] `uwufy_text()` implementation
  - [ ] Character replacement logic
  - [ ] Unicode handling

### Time/Date Utilities [ ]
- [ ] Create `src/utils/time.rs`
  - [ ] Timezone parsing and validation
  - [ ] Next trigger calculation
  - [ ] Discord timestamp formatting
  - [ ] Friday detection
  - [ ] 7:27 AM/PM calculation

### Welcome/Goodbye Message Utilities [x]
- [x] done subtasks redacted

### Reminder Utilities [ ]
- [ ] Create `src/utils/reminders.rs`
  - [ ] Message formatting helpers
  - [ ] Embed builders
  - [ ] Role mention formatting
  - [ ] DM sending helpers

---

## **Security & Validation**

### Input Validation [ ]
- [ ] Validate timezone inputs
- [ ] Validate time format (HH:MM)
- [ ] Validate role IDs exist
- [ ] Validate channel permissions
- [ ] Sanitize message content
- [ ] Validate embed data structure
- [ ] Validate welcome/goodbye message content
- [ ] Validate placeholder usage
- [ ] Validate embed URLs and dimensions

### Permission Checking [ ]
- [ ] Administrator permission for config writes
- [ ] Read-only access for members
- [ ] Bot permissions for channels
- [ ] Bot role hierarchy for pinging
- [ ] DM permissions for users
- [ ] Send message permissions for welcome/goodbye channels

### Error Handling [ ]
- [ ] Handle blocked DMs gracefully
- [ ] Handle deleted channels/roles
- [ ] Handle missing permissions
- [ ] Log all reminder failures
- [ ] Log welcome/goodbye message failures
- [ ] User-friendly error messages
- [ ] Handle member join/leave event errors

---

## **Testing Requirements**

### Unit Tests [ ]
- [x] Test about command functionality
- [ ] Test `src/tests/reminders_tests.rs`
  - [ ] Timezone calculation tests
  - [ ] Next trigger time calculation
  - [ ] WYSI 7:27 AM/PM tests
  - [ ] Femboy Friday midnight tests
- [ ] Test `src/tests/scheduler_tests.rs`
  - [ ] Due reminder detection
  - [ ] Execution logging
- [ ] Test `src/tests/user_settings_tests.rs`
  - [ ] Timezone validation
  - [ ] Subscription management
- [ ] Test `src/tests/welcome_goodbye_tests.rs`
  - [ ] Placeholder replacement functionality
  - [ ] Message formatting
  - [ ] Embed building
  - [ ] Member count accuracy
- [ ] Test uwufy functionality
- [ ] Test API integrations

### Integration Tests [ ]
- [ ] End-to-end reminder flow
- [ ] Subscription flow testing
- [ ] Permission checking
- [ ] Dashboard functionality
- [ ] Scheduler integration
- [ ] Welcome/goodbye message flow
- [ ] Member join/leave event handling

---

## **Performance & Optimization**

### Caching System [ ]
- [ ] Implement API response caching
- [ ] Cache user settings
- [ ] Cache guild configurations
- [ ] Cache welcome/goodbye configurations
- [ ] Scheduled cache cleanup

### Database Optimization [ ]
- [ ] Add indexes for reminder queries
- [ ] Add indexes for welcome/goodbye config lookups
- [ ] Optimize subscription lookups
- [ ] Clean up old reminder logs
- [ ] Connection pool tuning

### Scheduler Optimization [ ]
- [ ] Efficient due reminder queries
- [ ] Batch DM sending
- [ ] Parallel execution where possible
- [ ] Memory usage monitoring

---

## **Deployment & Production**

### Documentation [ ]
- [ ] Update README.md with full feature list
- [ ] Document timezone format
- [ ] Document reminder types
- [ ] Document welcome/goodbye placeholder variables
- [ ] API setup instructions

---

## **Future Enhancements (//TODO)**

### High Priority [ ]
- [ ] Custom reminders with cron scheduling
- [ ] Giphy API full integration
- [ ] Reminder templates
- [ ] Bulk subscription management
- [ ] Advanced welcome/goodbye message templates

### Medium Priority [ ]
- [ ] Reminder analytics dashboard
- [ ] Advanced scheduling options
- [ ] Welcome/goodbye message statistics
- [ ] Role-based welcome messages

### Low Priority [ ]
- [ ] Advanced role conditions
- [ ] Reminder categories/tags
- [ ] Welcome message DM option
- [ ] Advanced placeholder variables (user avatar, account age, etc.)

---
