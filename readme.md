# Pota Browser

Hello! This is a **very early alpha** version of a hypothetical profile manager and anti-detection browser written in Rust. It's completely open-source and experimental.

## 🚧 Still a Work in Progress!

I’m actively developing Pota Browser, and there’s still a lot to do. If you’re curious, feel free to check out the repo, test it out, and let me know what you think. **Bug reports, feature ideas, and contributions are more than welcome!** 🙌
## Join

- **IRC:** [irc.libera.chat #potabrowser](https://web.libera.chat/#potabrowser)  
- **0xchat Group:** `nostr:naddr1qpqrzct9xvmnywfc8qcnyc3jx3jrvdp5xycnvde5vycrze34xguxvvtx8yckyvnrxuexzdnrx4jkzcnyv33nvc3kvycxxcny8ymx2e33vgq3wamnwvaz7tm8wfhh2urn9cc8scmgv96zucm0d5pqqqcyqqqfskqjky3kl`

## Usage

### Requirements

- **Google Chrome** must be installed in the default path.
- **Rust** must be installed.
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

## Spoofing things
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

#### Intl.DateTimeFormat() Spoofing

Injectin this javascript seems works.

```
(function() {
  // Save the original method
  const originalResolvedOptions = Intl.DateTimeFormat.prototype.resolvedOptions;
  
  Intl.DateTimeFormat.prototype.resolvedOptions = function() {
    // Get the original options
    const options = originalResolvedOptions.apply(this, arguments);
    // Force the timeZone value
    options.timeZone = 'America/New_York';
    return options;
  };
})();
```

## Useful Chrome Arguments for Anti-Detection Mode

- **`--lang`** & **`--accept-lang`**  
  - These arguments override:  
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

- [ ] Automatic Chrome executable discovery.  
- [ ] WebRTC **fake** spoofing (correctly spoof host & STUN IP).  
- [ ] Spoofing all `navigator` properties (device, OS, hardware, browser, etc.).  
- [ ] Screen size, resolution, window, and viewport property spoofing.  
- [ ] Managing multiple CDP sessions.
- [ ] Many other things. 

## References

1: https://chebrowser.site/doc/en/profiles.html#webrtc-settings