# TickTick Today's Tasks

A Rust command-line application that fetches and displays your tasks due today from TickTick using the TickTick Open API.

## Features

- 🔐 OAuth2 authentication with TickTick
- 🌐 **Automatic callback server** - no manual code copying required!
- 📅 Fetches tasks due today across all projects
- 🎯 Displays task priorities with emoji indicators
- 📋 Shows subtasks and completion status
- ⏰ Displays due times and start times
- 🎨 Clean, formatted output
- 🚀 **Auto-opens browser** for seamless authentication

## Prerequisites

1. **TickTick Developer Account**: Register your application at the [TickTick Developer Center](https://developer.ticktick.com/)
2. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)

## Setup

1. **Clone and navigate to the project:**
   ```bash
   cd tick
   ```

2. **Get your TickTick API credentials:**
   - Visit the [TickTick Developer Center](https://developer.ticktick.com/)
   - Register a new application
   - Note down your `client_id` and `client_secret`
   - **Important**: Set the redirect URI to exactly: `http://localhost:8080/callback`

3. **Configure your credentials (choose one method):**

   ### Method 1: TOML Configuration File (Recommended)
   Create a configuration file in your home directory:
   ```bash
   # On Windows: C:\Users\YourUsername\.ticktick.toml
   # On macOS/Linux: ~/.ticktick.toml
   ```
   
   Content:
   ```toml
   [ticktick]
   client_id = "your_actual_client_id"
   client_secret = "your_actual_client_secret"
   
   # Optional: Custom redirect URI (defaults to http://localhost:8080/callback)
   # redirect_uri = "http://localhost:8080/callback"
   ```

   ### Method 2: Environment Variables
   ```bash
   cp .env.example .env
   # Edit .env with your actual credentials
   ```

## Usage

### Option 1: TOML Configuration (Recommended)
Create `~/.ticktick.toml` with your credentials and run:

```bash
cargo run
```

The program will automatically:
1. Load your credentials from the TOML file
2. Start a local web server on port 8080
3. Open your browser to the TickTick authorization page
4. Capture the authorization code automatically
5. Exchange it for an access token
6. Fetch and display your today's tasks

### Option 2: Interactive Mode
Simply run the program without any configuration:

```bash
cargo run
```

The program will:
1. Offer to create an example configuration file, or
2. Ask for your credentials manually
3. Follow the same OAuth flow as above

### Option 3: Environment Variables
Set up your `.env` file with your credentials and optionally an access token:

```bash
cargo run
```

The program will automatically use the environment variables if available.

## Configuration Priority

The program checks for credentials in the following order:

1. **Environment variables** (if all are set):
   - `TICKTICK_CLIENT_ID`
   - `TICKTICK_CLIENT_SECRET`
   - `TICKTICK_REDIRECT_URI`
   - `TICKTICK_ACCESS_TOKEN` (optional, skips OAuth if provided)

2. **TOML configuration file** (`~/.ticktick.toml`)

3. **Interactive input** (manual entry via prompts)

This allows you to use environment variables for CI/CD or temporary overrides while keeping your main configuration in the TOML file.

## Authentication Flow

1. **Local Callback Server**: The program starts a temporary web server on `localhost:8080`
2. **Browser Opens**: Your default browser opens to the TickTick authorization page
3. **User Authorizes**: You grant permission to the application
4. **Automatic Capture**: The callback server automatically captures the authorization code
5. **Token Exchange**: The code is exchanged for an access token
6. **Server Stops**: The temporary server shuts down
7. **Tasks Retrieved**: Your today's tasks are fetched and displayed

## Example Output

```
🎯 TickTick Today's Tasks
========================

Enter your client_id: your_client_id_here
Enter your client_secret: your_client_secret_here

🔗 Please visit this URL to authorize the application:
https://ticktick.com/oauth/authorize?...

🌐 Started local callback server on http://localhost:8080/callback
🌐 Waiting for authorization callback...
   (A browser window should open automatically, or copy the URL above)

✅ Received authorization code, exchanging for access token...
✅ Successfully obtained access token!

🗓️  Fetching today's tasks...

📋 Checking 3 projects for today's tasks...
  🔍 Checking project: Work Tasks
  🔍 Checking project: Personal
  � Checking project: Shopping

�📅 You have 2 task(s) for today:

┌─────────────────────────────────────────────────
│ 🔴 Complete project presentation
│ 📁 Project: Work Tasks
│ 📝 Content: Prepare slides for quarterly review
│ ⏰ Due: 17:00
│ 📋 Subtasks:
│   ☐ Create slides
│   ☐ Add charts
│   ✅ Review content
└─────────────────────────────────────────────────

┌─────────────────────────────────────────────────
│ 🟡 Buy groceries
│ 📁 Project: Personal
│ ⏰ Due: 19:00
└─────────────────────────────────────────────────
```

## Priority Indicators

- 🔴 High priority
- 🟡 Medium priority  
- 🔵 Low priority
- ⚪ No priority

## Task Detection

The program considers a task "due today" if:
- The task's due date is today
- The task's start date is today
- The task is not completed (status = 0)

## Security Notes

- Keep your `client_secret` secure and never commit it to version control
- Access tokens expire and you'll need to re-authenticate periodically
- The callback server only runs temporarily during authentication
- The `.env` file is gitignored by default

## Troubleshooting

### "No access token available"
Make sure you've completed the OAuth flow or set the `TICKTICK_ACCESS_TOKEN` environment variable.

### "Failed to exchange code for token"
Check that your `client_id`, `client_secret`, and redirect URI are correct and match what you configured in the TickTick Developer Center. The redirect URI must be exactly `http://localhost:8080/callback`.

### "Connection refused" or callback server issues
- Ensure port 8080 is not already in use
- Check your firewall settings allow local connections on port 8080
- Make sure the redirect URI in your TickTick app settings is exactly `http://localhost:8080/callback`

### "No tasks due today"
This means you either have no tasks scheduled for today, or all your today's tasks are already completed. Great job! 🎉

### Browser doesn't open automatically
Copy the authorization URL from the terminal and paste it into your browser manually.

## Dependencies

- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` - JSON serialization/deserialization
- `chrono` - Date and time handling
- `base64` - Base64 encoding for Basic Auth
- `url` - URL parsing and manipulation
- `anyhow` - Error handling
- `dotenv` - Environment variable loading
- `warp` - Web server for OAuth callback
- `toml` - TOML configuration file parsing
- `dirs` - Cross-platform directory locations

## Contributing

Feel free to submit issues and enhancement requests!

## License

This project is open source and available under the [MIT License](LICENSE).
