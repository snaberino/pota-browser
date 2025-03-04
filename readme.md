# Pota Browser

Hello! This is a **very early alpha** version of a hypothetical profile manager and anti-detection browser written in Rust. It's completely open-source and experimental.

The aim is to use the standard browser installation and tweak it in order to avoid fingerprint restriction and bypass antibot.

At the moment only Google chrome is supported.

## 🚧 Still a Work in Progress!

I’m actively developing Pota Browser, and there’s still a lot to do. If you’re curious, feel free to check out the repo, test it out, and let me know what you think. **Bug reports, feature ideas, and contributions are more than welcome!!** 🙌

![Latest Screenshot](assets/pota-browser-screenshot.png)
## Join

- **IRC:** [irc.libera.chat #potabrowser](https://web.libera.chat/#potabrowser)  
## Usage

### Requirements

- **Google Chrome** 
- **Rust** 
### Installation & Usage

1. Clone the repository:
```
git clone https://github.com/snaberino/pota-browser.git
```
2. Navigate to the cloned folder: 
```
cd pota-browser
``` 
1. Run the project:
```
cargo run
``` 

## Proxy Handling

Currently, proxy support is implemented by passing the `--proxy-server` argument to Chrome, which only accepts the `host:port` format. To handle authentication, username and password injection is done via Chrome DevTools Protocol (CDP).

This is first proxy implementation and in future may change.

## Spoofing *things*
### Spoofing user agent

**`--user-agent="custom_user_agent"`** this method changes the UA string in the HTTPS header, but it might not alter all JavaScript-exposed properties.

### WebRTC Spoofing

Need to implement several types of spoofing and blocking IP detection via WebRTC.

- **block**: completely disables WebRTC functionality.
  That mode will block totally WebRTC, with **`--webrtc-ip-handling-policy=disable_non_proxied_udp`** and **`--force-webrtc-ip-handling-policy`**

- **default**: spoofing and blocking IP detection via WebRTC is disabled. But if the site uses WebRTC, you will see a message about it.

- fake: allow to spoof the external IP address returned by STUN servers.

### Timezone Spoofing

Emulation.setTimezoneOverride
Date()
Date().getTimezoneOffset()
Emulation.setGeolocationOverride
Intl.DateTimeFormat()


## Useful Chrome Arguments

- **`--lang`** & **`--accept-lang`**  
  - These arguments override local language and accepted language  
    - `Worker.langs`  
    - `Navigator.lang`

- **`--debugging-port`**  
  - Enables the Chrome DevTools Protocol (CDP) port.

- **`--proxy-server`**  
  - Passes the proxy as `host:port`. Authentication is handled via CDP injection.

- **`--user-data-dir`**  
  - Specifies a path for a new Chrome profile folder. Multiple profiles can be created inside this directory, similar to standard Chrome behavior.

- **`--no-first-run`**  
  - Launches Chrome directly without showing the first-run setup prompts.
- --no-default-browser-check

- **`--no-default-browser-check`**  
  - Launches Chrome directly without showing the first-run setup prompts.

- **`--headless`**  
  - Runs Chrome in headless mode (no graphical interface).

### WebRTC args

- **`--webrtc-ip-handling-policy=disable_non_proxied_udp`**  
  - Override WebRTC IP handling policy, plus **`--force-webrtc-ip-handling-policy`** will disable WebRTC.

- **`--force-webrtc-ip-handling-policy`**  
  - Override WebRTC IP handling policy to mimic the behavior when WebRTC IP handling policy is specified in Preferences.

- **`--enable-webrtc-hide-local-ips-with-mdns`** Localhost IP
  - Starting from a recent version (around Chrome 92), the feature to hide local IP addresses via mDNS is enabled by default.


## TODO

- [x] Automatic Chrome executable discovery.  
- [ ] WebRTC **fake** spoofing (correctly spoof host & STUN IP).  
- [ ] Spoofing all `navigator` properties (device, OS, hardware, browser, etc.).  
- [ ] Screen size, resolution, window, and viewport property spoofing.  
- [ ] Managing multiple CDP sessions.
- [ ] Many other things. 

## References

1. https://chebrowser.site/doc/en/profiles.html#webrtc-settings
2. https://github.com/daijro/camoufox
3. https://github.com/MiddleSchoolStudent/BotBrowser


"{\"Accept-Language\":\"it-IT,it;q=0.9\",\"Sec-Ch-Device-Memory\":\"8\",\"Sec-Ch-Ua\":\"\\\"Not(A:Brand\\\";v=\\\"99\\\", \\\"Google Chrome\\\";v=\\\"133\\\", \\\"Chromium\\\";v=\\\"133\\\"\",\"Sec-Ch-Ua-Arch\":\"\\\"x86\\\"\",\"Sec-Ch-Ua-Full-Version\":\"\\\"133.0.6943.54\\\"\",\"Sec-Ch-Ua-Full-Version-List\":\"\\\"Not(A:Brand\\\";v=\\\"99.0.0.0\\\", \\\"Google Chrome\\\";v=\\\"133.0.6943.54\\\", \\\"Chromium\\\";v=\\\"133.0.6943.54\\\"\",\"Sec-Ch-Ua-Platform\":\"\\\"Windows\\\"\",\"Sec-Ch-Ua-Platform-Version\":\"\\\"10.0.0\\\"\",\"User-Agent\":\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36\"}"


"{\"Accept-Language\":\"it-IT,it;q=0.9\",\"Sec-Ch-Device-Memory\":\"8\",\"Sec-Ch-Ua\":\"\\\"Not(A:Brand\\\";v=\\\"99\\\", \\\"Google Chrome\\\";v=\\\"133\\\", \\\"Chromium\\\";v=\\\"133\\\"\",\"Sec-Ch-Ua-Arch\":\"\\\"x86\\\"\",\"Sec-Ch-Ua-Full-Version\":\"\\\"133.0.6943.99\\\"\",\"Sec-Ch-Ua-Full-Version-List\":\"\\\"Not(A:Brand\\\";v=\\\"99.0.0.0\\\", \\\"Google Chrome\\\";v=\\\"133.0.6943.99\\\", \\\"Chromium\\\";v=\\\"133.0.6943.99\\\"\",\"Sec-Ch-Ua-Platform\":\"\\\"macOS\\\"\",\"Sec-Ch-Ua-Platform-Version\":\"\\\"12.1.0\\\"\",\"User-Agent\":\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36\"}"

Fingerprint Constructor

In these section you can create custom fingerprint to spoof.

OS :