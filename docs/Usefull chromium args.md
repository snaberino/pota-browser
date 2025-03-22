- **`--lang`** & **`--accept-lang`**  
    Override local language and accepted language settings.
    
    Affects:
    - `Worker.langs`
    - `Navigator.lang`

- **`--debugging-port`**  
    Enables the Chrome DevTools Protocol (CDP) port.

- **`--proxy-server`**  
    Passes the proxy as `host:port`. Authentication is handled via CDP injection.

- **`--user-data-dir`**  
    Specifies a path for a new Chrome profile folder.
    
    Multiple profiles can be created inside this directory, similar to standard Chrome behavior.

- **`--no-first-run`**  
    Launches Chrome directly without showing the first-run setup prompts.
  
- **`--no-default-browser-check`**  
    Prevents Chrome from asking to be set as the default browser.

- **`--headless`**  
    Runs Chrome in headless mode (no graphical interface).

- **`--hide-crash-restore-bubble`**  
    Disables the session crash pop-up.


## WebRTC Flags

- **`--webrtc-ip-handling-policy=disable_non_proxied_udp`**  
    Override WebRTC IP handling policy.
    
    - Using **`--force-webrtc-ip-handling-policy`** will disable WebRTC.

- **`--force-webrtc-ip-handling-policy`**  
    Override WebRTC IP handling policy to mimic the behavior when WebRTC IP handling policy is specified in Preferences.

- **`--enable-webrtc-hide-local-ips-with-mdns`**  
    Localhost IP handling.
    
    - Starting from Chrome 92, the feature to hide local IP addresses via mDNS is enabled by default.