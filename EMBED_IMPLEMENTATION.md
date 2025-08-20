# Purpose-Built Discord Embedding Solution

## **Core Concept & How It Works**

Discord only creates embeds for videos under 10MB, but this can be bypassed using the Open Graph meta-tag system.
The solution creates HTML files that Discord parses as video embeds, making them equivalent to direct .mp4 embeds.

According to Discord developers, arbitrary iframes cannot be embedded—the only way to embed video is by providing a direct URL to the video file in the og:video:url meta-tag.

## **Implementation Architecture**

### **High-Level Flow:**
1. Bot receives `/video {video_link}` command
2. Bot generates HTML file with proper meta-tags pointing to the video
3. Bot serves HTML file from the existing web server endpoint
4. Bot responds with HTML URL instead of direct video URL
5. Discord parses HTML and creates playable video embed

### **Required Components:**

#### **1. Web Server Component (Embedded in Bot)**
This Rust bot already has an HTTP server to serve the generated HTML files using `axum` framework.

**Endpoints needed:**
- `GET /video/{filename}.html` - Serves generated HTML files

#### **2. HTML Template System**
Create a template that generates the required meta tags for Discord compatibility.

**Essential Meta Tags:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta property="og:type" content="video.other">
    <meta property="og:video:url" content="{VIDEO_URL}">
    <meta property="og:video:type" content="video/mp4">
    <meta property="og:video:width" content="{WIDTH}">
    <meta property="og:video:height" content="{HEIGHT}">
    <meta property="og:image" content="{THUMBNAIL_URL}">
    <meta property="og:title" content="{TITLE}">
    <meta property="og:description" content="{DESCRIPTION}">
    
    <!-- Twitter Card meta tags for better compatibility -->
    <meta name="twitter:card" content="player">
    <meta name="twitter:player" content="{VIDEO_URL}">
    <meta name="twitter:player:width" content="{WIDTH}">
    <meta name="twitter:player:height" content="{HEIGHT}">
    <meta name="twitter:image" content="{THUMBNAIL_URL}">
    
    <!-- Discord-specific optimization -->
    <meta name="theme-color" content="#FFFFFF">
</head>
<body>
    <video controls width="{WIDTH}" height="{HEIGHT}">
        <source src="{VIDEO_URL}" type="video/mp4">
    </video>
</body>
</html>
```

#### **3. File Management System**
Store generated HTML files with unique identifiers to avoid conflicts.

**Recommended Structure:**
```
/embed_files/
  ├── abc123.html
  ├── def456.html
  └── ...
```

## **Bot Command Implementation Details**

### **Command Handler Logic:**

1. **Parse video URL from command**
   - Validate URL format
   - Extract filename for title generation
   - Determine video dimensions (optional: use FFprobe or default to 1920x1080)

2. **Generate unique identifier**
   - Use random alphanumeric string (8-12 characters)
   - Ensure no filename conflicts

3. **Create HTML content**
   - Replace template placeholders with actual values
   - Use video URL directly for og:video:url

4. **Save HTML file**
   - Write to filesystem in designated directory
   - Ensure proper file permissions

5. **Construct response URL**
   - Format: `https://your-bot-domain.com/video/{unique_id}.html`
   - Return this URL as bot response

### **Video Metadata Extraction (Optional Enhancement)**

For better embeds, extract video information:
- **Duration**: For description text
- **Dimensions**: For accurate meta tag values
- **Thumbnail**: Generate or extract first frame
- **File size**: For validation

**Libraries to consider:**
- `ffmpeg-next` for video analysis
- `image` crate for thumbnail generation

### **Configuration Requirements**

**Environment Variables:**
- `EMBED_DIR`: Directory for storing HTML files
There are already some environment variables defined in `.env` file:
- WEB_HOST=127.0.0.1
- WEB_PORT=3000 
- BASE_URL=http://localhost:3000

**File Structure:**
```
src/
├── main.rs 
├── commands/
│   └── video.rs
├── web/
│   ├── server.rs
│   ├── templates.rs
│   └── mod.rs
└── utils/
    ├── file_manager.rs
    └── video_utils.rs
```

## **Discord Compatibility Specifications**

### **File Serving Requirements:**
- **CORS Headers**: Must include `Access-Control-Allow-Origin: *`
- **Content-Type**: HTML files must serve as `text/html`
- **Direct Access**: URLs must not require authentication

## **Error Handling & Edge Cases**

### **Input Validation:**
- Verify URL is accessible
- Check if URL points to supported video format

### **File Management:**
- Manage concurrent file creation
- Prevent directory traversal attacks

### **Web Server Considerations:**
- Proper HTTP status codes
- Graceful handling of missing files

## **Performance Optimizations**

### **Caching Strategy:**
- Cache generated HTML for identical video URLs
- Use video URL hash as cache key
- Implement TTL for cache entries

### **Async Operations:**
- Generate HTML files asynchronously
- Non-blocking file I/O operations
- Concurrent handling of multiple requests

## **Security Considerations**

### **Input Sanitization:**
- Sanitize video URLs before embedding
- Validate domain allowlists if needed
- Prevent HTML injection in generated content

## **Deployment Notes**

### **Domain Setup:**
It will be localhost for now, but it needs to be easily changeable for later.

### **Reverse Proxy Configuration:**
If using nginx/caddy in front of your bot:
```
location ~ /[a-zA-Z0-9]+\.html$ {
    proxy_pass http://localhost:8080;
    add_header Access-Control-Allow-Origin *;
}
```
