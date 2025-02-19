# Pota Browser

Hello! This is a **very early alpha** version of a hypothetical profile manager and anti-detection browser written in Rust. It's completely open-source and experimental.
## Join

- **IRC:** [irc.libera.chat #potabrowser](https://web.libera.chat/#potabrowser)  
- **0xchat Group:** `nostr:naddr1qpqrzct9xvmnywfc8qcnyc3jx3jrvdp5xycnvde5vycrze34xguxvvtx8yckyvnrxuexzdnrx4jkzcnyv33nvc3kvycxxcny8ymx2e33vgq3wamnwvaz7tm8wfhh2urn9cc8scmgv96zucm0d5pqqqcyqqqfskqjky3kl`

## Proxy Handling

Currently, proxy support is implemented by passing the `--proxy-server` argument to Chrome, which only accepts the `host:port` format. To handle authentication, username and password injection is done via Chrome DevTools Protocol (CDP).

WebRTC Spoofing

Timezone Spoofing

Emulation.setTimezoneOverride
Date()
Date().getTimezoneOffset()
Emulation.setGeolocationOverride

Intl.DateTimeFormat() Spoofing

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

- **`--headless`**  
  - Runs Chrome in headless mode (no graphical interface).

- **`--webrtc-ip-handling-policy`**  
  - Override WebRTC IP handling policy to mimic the behavior when WebRTC IP handling policy is specified in Preferences.

- **`--force-webrtc-ip-handling-policy`**  
  - Override WebRTC IP handling policy to mimic the behavior when WebRTC IP handling policy is specified in Preferences.
## TODO

- [ ] Automatic Chrome executable discovery.  
- [ ] WebRTC spoofing (correctly spoof host & STUN IP).  
- [ ] Spoofing all `navigator` properties (device, OS, hardware, browser, etc.).  
- [ ] Screen size, resolution, window, and viewport property spoofing.  
