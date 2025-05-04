# Pota Browser

**Pota Browser** is an experimental, open-source **profile manager** for Chromium-based browsers, written in Rust. It enables advanced control over browser profiles and proxy routing, with a focus on privacy, automation, and anti-detection techniques.

## 🚧 Still a Work in Progress!

I’m actively developing Pota Browser, and there’s still a lot to do. If you’re curious, feel free to check out the repo, test it out, and let me know what you think. **Bug reports, feature ideas, and contributions are more than welcome!!** 🙌

## Read about

- [Spoofing Things](https://github.com/snaberino/pota-browser/blob/master/docs/Spoofing%20things.md)
- [Usefull Chromium args](https://github.com/snaberino/pota-browser/blob/master/docs/Usefull%20chromium%20args.md)

## Usage

### Requirements

- **Chromium based browser**  
- **Rust**

### Installation & Usage

1. Clone the repository:
   ```bash
   git clone https://github.com/snaberino/pota-browser.git
   ```
2. Navigate to the cloned folder: 
   ```bash
   cd pota-browser
   ``` 
3. Run the project:
   ```bash
   cargo run
   ```

## Functions 

- Manage multiple isolated Chromium profiles
- Custom profile paths per instance
- Support for custom Chromium installations
- Built-in proxy routing with local proxy client
- HTTP/SOCKS proxy support
- Proxy manager

## Proxy Handling

Pota implements a **proxy chain**: it spins up a **local proxy client** that connects to a **remote proxy**, and then launches Chromium to connect to the local endpoint (e.g. `127.0.0.1:PORT`).

This provides:
- Better control over proxy behavior
- Support for authenticated proxies
- Decoupled browser ↔ proxy logic

## TODO

- [x] Socks proxy support  
- [ ] WebRTC **fake** spoofing (correctly spoof host & STUN IP)  
- [ ] Spoofing all `navigator` properties (device, OS, hardware, browser, etc.)  
- [ ] Screen size, resolution, window, and viewport property spoofing  
- [ ] Many other things  

## 🧠 A Note on Anti-Detection

For a **complete anti-detect browser experience**, it's not enough to just tweak launch flags or DevTools overrides. You really need to patch Chromium’s **C++ source code** to deeply spoof canvas, WebGL, audio, and more.

That’s out of scope for this project **for now** — but if you’re comfortable diving into Chromium’s internals and love hacking on C++... **help is very welcome!** 😄

## References

1. https://chebrowser.site/doc/en/profiles.html#webrtc-settings  
2. https://github.com/daijro/camoufox  
3. https://github.com/MiddleSchoolStudent/BotBrowser
