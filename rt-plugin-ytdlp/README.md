# YouTube Downloader Plugin (yt-dlp)

## Overview

The YouTube Downloader Plugin (`rt-plugin-ytdlp`) is a comprehensive media downloading plugin for Rust Toolbox that integrates `yt-dlp` functionality. It enables users to download videos and audio content from YouTube and hundreds of other supported websites with extensive format and quality options.

## Features

### Core Functionality
- **Single Video Downloads**: Download individual videos from supported platforms
- **Playlist Support**: Download entire playlists with batch processing
- **Format Selection**: Choose from available video/audio formats and qualities
- **Subtitle Downloads**: Download subtitles including auto-generated captions
- **Custom Output**: Configurable output directories and filename templates
- **Progress Tracking**: Real-time download progress monitoring
- **Error Recovery**: Robust error handling and retry mechanisms

### Supported Platforms
The plugin supports all websites that `yt-dlp` supports, including:
- **Video Platforms**: YouTube, Vimeo, Dailymotion, Twitch
- **Social Media**: Twitter, Facebook, Instagram, TikTok
- **Chinese Platforms**: Bilibili, 优酷 (Youku), 腾讯视频 (Tencent Video), 爱奇艺 (iQiyi)
- **Audio Platforms**: SoundCloud, Spotify (metadata), Bandcamp
- **News/Educational**: BBC iPlayer, Khan Academy, Coursera
- **Live Streaming**: YouTube Live, Twitch streams

For the complete list, see [yt-dlp supported sites](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md).

## Installation

### Prerequisites
1. **Rust Toolbox**: Requires rt-cli or rt-gui host application
2. **yt-dlp**: Must be installed and available in system PATH
   - Install via pip: `pip install yt-dlp` (requires Python 3.7+)
   - Or download binary from [yt-dlp releases](https://github.com/yt-dlp/yt-dlp/releases)
3. **FFmpeg** (Optional): Required for format conversion and merging
   - Download from [FFmpeg website](https://ffmpeg.org/download.html)
   - Or install via package manager: `apt install ffmpeg` / `brew install ffmpeg`

### Build Instructions

1. **Clone Repository**:
   ```bash
   git clone https://github.com/your-repo/rust-tool.git
   cd rust-tool
   ```

2. **Build Plugin**:
   ```bash
   cargo build --release --package rt-plugin-ytdlp
   ```

3. **Deploy Plugin**:
   ```bash
   # Copy to plugins directory
   cp target/release/rt-plugin-ytdlp.exe plugins/  # Windows
   cp target/release/rt-plugin-ytdlp plugins/      # Unix systems
   ```

### Verification
```bash
# Verify yt-dlp is available
yt-dlp --version

# Verify plugin is loaded
rt-cli list | grep ytdlp

# Get plugin specification
rt-cli plugin spec --name media.ytdlp
```

## Usage

### Command Line Interface (CLI)

#### Basic Video Download
```bash
echo '{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "format": "best",
  "output_dir": "./downloads"
}' | rt-cli run media.ytdlp
```

#### Audio-Only Download
```bash
echo '{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "format": "bestaudio",
  "output_dir": "./music",
  "filename_template": "%(artist)s - %(title)s.%(ext)s"
}' | rt-cli run media.ytdlp
```

#### Playlist Download
```bash
echo '{
  "url": "https://www.youtube.com/playlist?list=PLrAXtmRdnEQy6nuLMt9JVcwbFBbavGZCh",
  "playlist": true,
  "format": "best[height<=720]",
  "output_dir": "./playlist"
}' | rt-cli run media.ytdlp
```

#### Download with Subtitles
```bash
echo '{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "format": "best",
  "subtitles": true,
  "subtitle_langs": ["en", "zh-CN"],
  "output_dir": "./videos"
}' | rt-cli run media.ytdlp
```

### Graphical Interface (GUI)

1. **Launch GUI**: Start `rt-gui` application
2. **Navigate to Media Tools**: Find "YouTube Downloader" in the media section
3. **Configure Download**:
   - Enter video/playlist URL
   - Select desired format and quality
   - Choose output directory
   - Configure additional options (subtitles, filename template)
4. **Execute**: Click "Run" to start download
5. **Monitor Progress**: View download progress and results in output panel

## Input/Output Schema

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "url": {
      "type": "string",
      "title": "Video URL",
      "description": "URL of the video or playlist to download"
    },
    "format": {
      "type": "string",
      "default": "best",
      "title": "Format",
      "description": "Video/audio format selection"
    },
    "playlist": {
      "type": "boolean",
      "default": false,
      "title": "Download Playlist",
      "description": "Whether to download entire playlist"
    },
    "subtitles": {
      "type": "boolean",
      "default": false,
      "title": "Download Subtitles",
      "description": "Whether to download subtitle files"
    },
    "subtitle_langs": {
      "type": "array",
      "items": {"type": "string"},
      "title": "Subtitle Languages",
      "description": "List of subtitle language codes to download"
    },
    "output_dir": {
      "type": "string",
      "default": ".",
      "title": "Output Directory",
      "description": "Directory to save downloaded files"
    },
    "filename_template": {
      "type": "string",
      "default": "%(title)s.%(ext)s",
      "title": "Filename Template",
      "description": "Template for output filenames"
    },
    "quality": {
      "type": "string",
      "title": "Quality Preference",
      "description": "Preferred video quality (e.g., 720p, 1080p)"
    }
  },
  "required": ["url"]
}
```

#### Field Descriptions
- **url**: Video or playlist URL (required)
- **format**: Format selector (best, worst, bestvideo+bestaudio, etc.)
- **playlist**: Download entire playlist if URL is a playlist
- **subtitles**: Download available subtitle files
- **subtitle_langs**: Specific subtitle languages to download
- **output_dir**: Target directory for downloaded files
- **filename_template**: Custom filename pattern using yt-dlp variables
- **quality**: Quality preference filter

### Output Schema
```json
{
  "type": "object",
  "properties": {
    "success": {
      "type": "boolean",
      "title": "Download Success",
      "description": "Whether the download completed successfully"
    },
    "files": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "path": {"type": "string"},
          "size": {"type": "integer"},
          "format": {"type": "string"},
          "duration": {"type": "number"}
        }
      },
      "title": "Downloaded Files",
      "description": "List of successfully downloaded files"
    },
    "message": {
      "type": "string",
      "title": "Status Message",
      "description": "Human-readable status or error message"
    },
    "metadata": {
      "type": "object",
      "title": "Video Metadata",
      "description": "Extracted video information"
    }
  },
  "required": ["success", "files", "message"]
}
```

#### Result Fields
- **success**: Boolean indicating overall operation success
- **files**: Array of downloaded files with metadata
- **message**: Localized status or error message
- **metadata**: Video information (title, uploader, duration, etc.)

## Format Selection

### Common Format Selectors
- **`best`**: Best quality available (default)
- **`worst`**: Lowest quality available
- **`bestvideo+bestaudio`**: Best video and audio streams merged
- **`bestaudio`**: Best audio-only stream
- **`best[height<=720]`**: Best quality up to 720p
- **`worst[filesize<50M]`**: Smallest file under 50MB

### Quality Filters
- **Height**: `best[height<=1080]`, `worst[height>=480]`
- **File Size**: `best[filesize<100M]`, `worst[filesize>10M]`
- **Format**: `best[ext=mp4]`, `bestaudio[ext=m4a]`
- **Codec**: `best[vcodec=h264]`, `bestaudio[acodec=aac]`

### Advanced Examples
```bash
# Best MP4 video under 100MB
"format": "best[ext=mp4][filesize<100M]"

# Audio-only in MP3 format
"format": "bestaudio[ext=m4a]/bestaudio"

# 720p video with AAC audio
"format": "best[height=720][acodec=aac]"
```

## Filename Templates

### Template Variables
- **`%(title)s`**: Video title
- **`%(uploader)s`**: Channel/uploader name
- **`%(upload_date)s`**: Upload date (YYYYMMDD)
- **`%(duration)s`**: Video duration in seconds
- **`%(view_count)s`**: View count
- **`%(like_count)s`**: Like count
- **`%(ext)s`**: File extension

### Template Examples
```bash
# Channel and title
"%(uploader)s - %(title)s.%(ext)s"

# Date and title
"%(upload_date)s - %(title)s.%(ext)s"

# Organized by uploader
"%(uploader)s/%(title)s.%(ext)s"

# With quality info
"%(title)s [%(height)sp].%(ext)s"
```

## Configuration

### Environment Variables
- **`YTDLP_PATH`**: Custom path to yt-dlp executable
- **`FFMPEG_PATH`**: Custom path to FFmpeg executable
- **`YTDLP_CONFIG`**: Path to yt-dlp configuration file

### Performance Tuning
- **Concurrent Downloads**: Adjust based on network capacity
- **Rate Limiting**: Respect site rate limits to avoid blocking
- **Retry Logic**: Configure retry attempts for failed downloads
- **Timeout Settings**: Set appropriate timeouts for slow connections

## Error Handling

### Common Error Conditions
- **URL Not Supported**: Site not supported by yt-dlp
- **Video Unavailable**: Private, deleted, or geo-blocked content
- **Format Not Available**: Requested format doesn't exist
- **Network Issues**: Connection timeouts or interruptions
- **Disk Space**: Insufficient storage for downloads
- **Permission Errors**: Cannot write to output directory

### Error Recovery
- **Automatic Retries**: Configurable retry attempts for transient failures
- **Partial Downloads**: Resume interrupted downloads when possible
- **Graceful Degradation**: Fall back to alternative formats
- **Detailed Logging**: Comprehensive error reporting for troubleshooting

## Internationalization

### Supported Languages
- **English (`en`)**: Primary development language
- **Chinese (`zh-CN`)**: Simplified Chinese translations

### Localized Elements
- **Tool Name**: "YouTube Downloader" / "YouTube下载器"
- **Field Labels**: Input/output field titles and descriptions
- **Status Messages**: Download progress and completion messages
- **Error Messages**: Localized error descriptions
- **User Guide**: Comprehensive usage instructions

## Development

### Project Structure
```
rt-plugin-ytdlp/
├── Cargo.toml              # Dependencies and metadata
├── README.md               # This documentation
├── input.json              # Sample input for testing
├── src/
│   ├── main.rs            # Plugin entry point and CLI handling
│   └── i18n.rs            # Internationalization support
└── locales/               # Internationalization resources
    ├── tool.en.json       # English translations
    └── tool.zh.json       # Chinese translations
```

### Dependencies
- **tokio**: Async runtime for process execution
- **serde**: JSON serialization/deserialization
- **clap**: Command-line argument parsing
- **rt-core**: Rust Toolbox core types and traits
- **log**: Logging framework

### Building and Testing
```bash
# Development build
cargo build --package rt-plugin-ytdlp

# Release build
cargo build --release --package rt-plugin-ytdlp

# Run tests
cargo test --package rt-plugin-ytdlp

# Integration test with sample input
echo '{"url": "https://www.youtube.com/watch?v=BaW_jenozKc", "format": "worst"}' | \
  cargo run --bin rt-plugin-ytdlp run
```

## Performance Optimization

### Download Speed
- **Parallel Downloads**: Multiple concurrent downloads for playlists
- **Fragment Downloads**: Parallel downloading of video segments
- **Network Optimization**: Optimal connection pooling and reuse
- **Bandwidth Management**: Configurable rate limiting

### Resource Usage
- **Memory Efficiency**: Streaming downloads without loading entire files
- **Disk I/O**: Efficient file writing and temporary file management
- **CPU Usage**: Minimal processing overhead during downloads
- **Network Efficiency**: Optimal request patterns and caching

## Security Considerations

### Safe Downloads
- **URL Validation**: Verify URLs before processing
- **Path Sanitization**: Prevent directory traversal attacks
- **File Type Validation**: Ensure downloaded files match expected types
- **Size Limits**: Configurable maximum file sizes

### Privacy Protection
- **No Data Collection**: Plugin doesn't collect or transmit user data
- **Local Processing**: All operations performed locally
- **Secure Cleanup**: Temporary files properly cleaned up
- **User Control**: Full user control over download locations and metadata

## Troubleshooting

### Common Issues

#### yt-dlp Not Found
- **Cause**: yt-dlp not installed or not in PATH
- **Solution**: Install yt-dlp and ensure it's accessible: `pip install yt-dlp`

#### Download Failures
- **Cause**: Video unavailable, network issues, or format problems
- **Solution**: Check URL validity, try different formats, verify network connection

#### Permission Errors
- **Cause**: Cannot write to output directory
- **Solution**: Ensure output directory exists and has write permissions

#### Slow Downloads
- **Cause**: Network limitations or server throttling
- **Solution**: Try different quality settings, check network connection

### Debug Information
```bash
# Enable verbose logging
RUST_LOG=debug rt-cli run media.ytdlp < input.json

# Test yt-dlp directly
yt-dlp --list-formats "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Check plugin specification
rt-cli plugin spec --name media.ytdlp --verbose
```

### Performance Monitoring
```bash
# Monitor download progress
tail -f ~/.rt-toolbox/logs/ytdlp.log

# Check system resources during download
htop  # or Task Manager on Windows
```

## License

This plugin is licensed under the same terms as the Rust Toolbox project. See the main project LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure all changes include:

- **Tests**: Comprehensive test coverage for new features
- **Documentation**: Updated documentation for changes
- **Localization**: Updates to all supported language files
- **Performance**: Consider performance impact of changes
- **Security**: Review security implications of modifications

## Support

For issues, questions, or contributions:

1. **GitHub Issues**: Report bugs and request features
2. **Documentation**: Check yt-dlp documentation for format questions
3. **Community**: Join project discussions and forums
4. **Testing**: Provide sample URLs that cause issues

## Changelog

### v0.1.0 (Current)
- Initial yt-dlp integration implementation
- Support for video and audio downloads
- Playlist download capabilities
- Subtitle download support
- Comprehensive format selection
- Full internationalization support
- CLI and GUI integration
- Extensive documentation and testing

### Future Enhancements
- **Progress Callbacks**: Real-time download progress reporting
- **Advanced Filtering**: More sophisticated content filtering options
- **Batch Operations**: Enhanced batch download management
- **Custom Extractors**: Support for additional site extractors
- **Download Queues**: Managed download queue system
- **Metadata Enhancement**: Extended metadata extraction and processing

---

**Happy downloading! 🎬🎵**