## Spoofing HTTP User-Agent

**`--user-agent="custom_user_agent"`** this method changes the UA string in the HTTPS header, but it might not alter all JavaScript-exposed properties.

## WebRTC Spoofing

Need to implement several types of spoofing and blocking IP detection via WebRTC.

- **block**: completely disables WebRTC functionality.
  That mode will block totally WebRTC, with **`--webrtc-ip-handling-policy=disable_non_proxied_udp`** and **`--force-webrtc-ip-handling-policy`**

- **default**: spoofing and blocking IP detection via WebRTC is disabled. But if the site uses WebRTC, you will see a message about it.

- fake: allow to spoof the external IP address returned by STUN servers.

## Timezone Spoofing

Emulation.setTimezoneOverride
Date()
Date().getTimezoneOffset()
Emulation.setGeolocationOverride
Intl.DateTimeFormat()