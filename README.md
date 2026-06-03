# ShadowGuessr 🌍🕵️‍♂️

**ShadowGuessr** is an external, real-time telemetry interceptor and radar overlay for GeoGuessr. 
It works by passively sniffing local WebSocket traffic, unmasking and decompressing binary payloads on 
the fly, and rendering an interactive map overlay.

> [!WARNING]
> This project was created **strictly for educational purposes, protocol 
> research, and personal use.** I am strongly against cheating in multiplayer games. This 
> tool is designed to study WebSocket packet structures, `Per-Message Deflate` compression algorithms, 
> and network telemetry. **Do not use this tool to gain an unfair advantage in 
> ranked, competitive, or public matchmaking games.**

## 🚀 Features
* **Zero-Injection:** Operates entirely outside the browser/game client using packet sniffing (`tshark`).
* **Real-Time Telemetry:** Parses WebSocket payloads containing compressed & masked useful data.
* **State Tracking:** Keeps track of the current round, player health, multipliers, and match state.
* **Opponent Tracking:** Intercepts and displays opponent pin placements and movements.

## 🛠️ Technologies
* **Language:** [Rust](https://www.rust-lang.org/)
* **Frontend/UI:** [egui](https://github.com/emilk/egui) / `eframe`
* **Map Rendering:** [walkers](https://github.com/podusowski/walkers)
* **Network Sniffing:** [Tshark](https://www.wireshark.org/docs/man-pages/tshark.html) (Wireshark CLI)

---

## ⚙️ Prerequisites

1. **Rust Toolchain:** Install from [rustup.rs](https://rustup.rs/).
2. **Tshark:** Must be installed and accessible in your system's `PATH`.
    * *Linux:* `sudo apt install tshark` (Ensure your user is in the `wireshark` group to capture packets without root).
    * *Windows:* Install [Wireshark](https://www.wireshark.org/) and ensure `tshark.exe` is in your environment variables.
3. **Google Maps Static API Key:** Required to resolve coordinates for custom user photospheres (pano IDs) when the game doesn't explicitly send them.

## 🔑 Obtaining the API Key
ShadowGuessr uses the Google Maps Static API to fetch coordinates based on Panorama IDs.
1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Create a new project or select an existing one.
3. Navigate to **APIs & Services > Library**.
4. Search for **Maps Static API** and click **Enable**.
5. Go to **APIs & Services > Credentials** and click **Create Credentials > API Key**.
6. *Recommendation:* Restrict your API key to prevent unauthorized usage.
7. Paste this key into the app's settings or configuration file.

---

## 🔓 Decrypting TLS Traffic (The Magic)

Because GeoGuessr runs over HTTPS/WSS, ShadowGuessr needs the TLS session keys to decrypt the traffic locally. 
We do this using the `SSLKEYLOGFILE` environment variable.

### If playing via Steam (GeoGuessr App)
Right-click GeoGuessr in your Steam Library -> Properties -> General -> Launch Options, and paste the following:
```bash
env SSLKEYLOGFILE="/tmp/shadow_keys.log" %command%
```

### If playing via Browser (Chrome/Brave/Edge)

You must launch your browser completely closed, then open it from the terminal with the environment variable attached:

- **Linux:** `SSLKEYLOGFILE="/tmp/shadow_keys.log" google-chrome`
- **Windows (PowerShell):** 
    ```powershell
    $env:SSLKEYLOGFILE="C:\temp\shadow_keys.log"
    Start-Process "chrome.exe"
    ```

> [!NOTE]
> Ensure the file path specified in your browser/Steam matches the path from the settings!

## 🏃‍♂️ Running the Project

1. Clone the repository: 
    ```bash
    git clone [https://github.com/xairaven/shadowguessr.git](https://github.com/xairaven/shadowguessr.git)
    cd shadowguessr
    ```
2. Start the game or browser with the `SSLKEYLOGFILE` variable.
3. Run the application: 
    ```bash
    cargo run --release 
   ```
4. Join a game. ShadowGuessr will automatically intercept the `SubscribeToLobby` 
and `DuelStarted` events and initialize the radar.

## 🤝 Contributing

Contributions, bug reports, and pull requests are welcome! If you find new undocumented event types in the protocol, 
feel free to open an issue. Please adhere to the `CODE_OF_CONDUCT.md`.

## 📄 License

This project is licensed under the MIT License - see the `LICENSE` file for details.