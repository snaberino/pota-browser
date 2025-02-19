# Pota Browser

Hello! This is a **very early alpha** version of a hypothetical profile manager and anti-detection browser written in Rust. It's completely open-source and experimental.
## Join

- **IRC:** [irc.libera.chat #potabrowser](https://web.libera.chat/#potabrowser)  
- **0xchat Group:** `nostr:naddr1qpqrzct9xvmnywfc8qcnyc3jx3jrvdp5xycnvde5vycrze34xguxvvtx8yckyvnrxuexzdnrx4jkzcnyv33nvc3kvycxxcny8ymx2e33vgq3wamnwvaz7tm8wfhh2urn9cc8scmgv96zucm0d5pqqqcyqqqfskqjky3kl`

## Proxy Handling

Currently, proxy support is implemented by passing the `--proxy-server` argument to Chrome, which only accepts the `host:port` format. To handle authentication, username and password injection is done via Chrome DevTools Protocol (CDP).

## Spoofing things

### WebRTC Spoofing

Need to implement several types of spoofing and blocking IP detection via WebRTC. See [[#^b11322]]

- **block**: completely disables WebRTC functionality.
  That mode will block totally WebRTC, with two args [[#^539353]] [[#^cc513e]]
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

- **`--headless`**  
  - Runs Chrome in headless mode (no graphical interface).

### WebRTC args

- **`--webrtc-ip-handling-policy=disable_non_proxied_udp`**   ^539353
  - Override WebRTC IP handling policy, plus **`--force-webrtc-ip-handling-policy`** will disable WebRTC.

- **`--force-webrtc-ip-handling-policy`**   ^cc513e
  - Override WebRTC IP handling policy to mimic the behavior when WebRTC IP handling policy is specified in Preferences.

- **`--enable-webrtc-hide-local-ips-with-mdns`** Localhost IP
  - Starting from a recent version (around Chrome 92), the feature to hide local IP addresses via mDNS is enabled by default.

## TODO

- [ ] Automatic Chrome executable discovery.  
- [ ] WebRTC spoofing (correctly spoof host & STUN IP).  
- [ ] Spoofing all `navigator` properties (device, OS, hardware, browser, etc.).  
- [ ] Screen size, resolution, window, and viewport property spoofing.  

## References

1: https://chebrowser.site/doc/en/profiles.html#webrtc-settings ^b11322