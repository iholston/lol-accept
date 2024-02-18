<h1>
  LoL-Play
</h1>

<p>
LoL-Play is a Windows tray application that will always Auto Accept your League of Legends games 🕹️☕ </br>
Doesn't use your 🖱️ or ⌨️ and works even if league is minimized or in the background </br></br>
</p>

</br>
<p align="left">
  <img src="https://github.com/iholston/lol-play/assets/32341824/06e5b20a-9a11-4f17-8e89-3847801e44df">
</p>

## Getting Started
- Download the [latest release](https://github.com/iholston/lol-play/releases)
- Double click on the .exe
- An icon will appear in the tray, right click to pause or exit 👍

## Development
- Install [Python](https://www.python.org/downloads/)
- Clone or download the repo
- ```pip install -r requirements.txt```

## Packaging to .exe
- ```pip install pyinstaller```
- ```pyinstaller --onefile --noconsole --name lol-play --add-data "icon.png:." main.py```

## Disclaimer
This program isn’t endorsed by Riot Games and doesn’t reflect the views or opinions of Riot Games or anyone officially involved in producing or managing League of Legends. League of Legends and Riot Games are trademarks or registered trademarks of Riot Games, Inc. League of Legends © Riot Games, Inc.
