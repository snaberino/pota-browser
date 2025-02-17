Hello, this is a very crappy a super alpha version of an ipotetic profile manager and anti detection browser written in rust and totally open source.

# Chrome usefull args for anti detection mode

--lang
--accept-lang

    with these two args we can override:

    Worker.langs
    Navigator.lang

--debugging-port

    In order to active di CDP port.

--proxy-server

    Pass host:port of the proxy, in order to handle authetication i use injecting auth with CDP.

--user-data-dir
    
    Set a path for a new chrome folder, inside that one could be possibile make others profiles. Like a normal Chrome.

--no-first-run

    This will open Chrome straight to operative status, avoiding initial popups for new profiles.

--headless

    Chrome will start in headless mode
    
# TODO

- [ ] Automatic chrome executable discovery.
- [ ] Spoofing WebRTC (spoof Host & STUN ip correctly).
- [ ] All navigator properties (device, OS, hardware, browser, etc.)
- [ ] Screen size, resolution, window, & viewport properties.