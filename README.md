<h1 align="center">
FossModManager
</h1>
<p align="center">
  <img src="https://github.com/user-attachments/assets/6745e82e-d1c9-4e2b-a66d-af06d364b485" alt="FossModManager Demo">
</p>

FossModManager is a local, offline mod manager for Linux, that is **nowhere close to being finished**. But you can still use it to manage and keep track of your installed mods, toggle exisiting mods on and off, delete and install new mods without having to use wine or manually installing every mod and skin you come across. 

I may continue to work on this, but this was intended to just be a proof of concept.

## What is this?!?!

- This was originally intended to be a mod manager for multiple games, but after starting on one game and realizing that was a huge mistake - so I switched to focus on a game I've been playing recently that has a pretty large modding community(also unfortunately what appears to be [a potentially large & looming banwave...just a heads up)](https://store.steampowered.com/news/app/2246340/view/534347841813873218)
- Install mods via .zip! Unfortunately no RAR support at the moment because I wasn't able to get unrar or xcompress to play nicely with zip/tauri
- See all of your installed mods and skins immediately! (after pointing the modmanager to your MonsterHunterWilds.exe) 
- That's pretty much it for now! There's not much else to do for an offline mod manager. If you're running a debian or rpm-based distro please give it a try just to let me know whether or not it works for you. 🙏

Place your skins in the following directory:

```sh
/path/to/your/root/game/directory/fossmodmanager/mods
```

Most skins will include two subdirectories modname_tex modname_model, be sure to bring both up into the mods directory and enable both to enable the skin.

## Distribution

I have binaries built via tarui-build [in the release section](https://github.com/slbillups/fossmodmanager-MHW/releases/tag/pre-release), if you aren't going to fork/clone/contribute I hope you give these a try

Otherwise just clone, and 
```sh
pnpm i && pnpm tauri dev
```

- The search feature is currently bare bones. You'll need to provide your own [Nexus Mods developer API key](https://www.nexusmods.com/users/myaccount?tab=api%20access) and place it in the root directory in an .env file with the key NEXUS_API_KEY=your_key_here.

- By default, opening the search window will only display the top 10 Monster Hunter Wilds mods.

- If things stop working, or mods aren't updating/the cache is invalidated - use the nuke button at the bottom of settings to clear the cache/config directories which should set you back to the setup page.

## Building / Recommended IDE Setup

- Your IDE of choice + rust/cargo [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://rust-analyzer.github.io/book/)

## License

This project is licensed under GPLv3. 

### Fonts

This project uses the **Crimson Text** font, licensed under the [SIL Open Font License, Version 1.1](https://scripts.sil.org/OFL).
