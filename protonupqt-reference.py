def get_steam_vdf_compat_tool_mapping(vdf_file: dict) -> dict:

    s = vdf_file.get('InstallConfigStore', {}).get('Software', {})

    # Sometimes the key is 'Valve', sometimes 'valve', see #226
    c = s.get('Valve') or s.get('valve')
    if not c:
        print('Error! config.vdf InstallConfigStore.Software neither contains key "Valve" nor "valve" - config.vdf file may be invalid!')
        return {}

    m = c.get('Steam', {}).get('CompatToolMapping', {})

    if not m:  # equal to m == {} , may occur after fresh Steam installation
        print('Warning: CompatToolMapping is empty')

    return m


def get_steam_app_list(steam_config_folder: str, cached=False, no_shortcuts=False) -> list[SteamApp]:
    """
    Returns a list of installed Steam apps and optionally game names and the compatibility tool they are using
    steam_config_folder = e.g. '~/.steam/root/config'
    Return Type: list[SteamApp]
    """
    global _cached_app_list

    if cached and _cached_app_list != []:
        return _cached_app_list

    libraryfolders_vdf_file = os.path.join(os.path.expanduser(steam_config_folder), 'libraryfolders.vdf')
    config_vdf_file = os.path.join(os.path.expanduser(steam_config_folder), 'config.vdf')

    apps = []

    try:
        v = vdf_safe_load(libraryfolders_vdf_file)
        c = get_steam_vdf_compat_tool_mapping(vdf_safe_load(config_vdf_file))

        for fid in v.get('libraryfolders'):
            if 'apps' not in v.get('libraryfolders').get(fid):
                continue
            fid_path = v.get('libraryfolders').get(fid).get('path')
            fid_libraryfolder_path = fid_path
            if fid == '0':
                fid_path = os.path.join(fid_path, 'steamapps', 'common')
            for appid in v.get('libraryfolders').get(fid).get('apps'):
                # Skip if app isn't installed to `/path/to/steamapps/common` - Skips soundtracks
                fid_steamapps_path = os.path.join(fid_libraryfolder_path, 'steamapps')  # e.g. /home/gaben/Games/steamapps
                appmanifest_path = os.path.join(fid_steamapps_path, f'appmanifest_{appid}.acf')
                if os.path.isfile(appmanifest_path):
                    appmanifest_install_path = vdf_safe_load(appmanifest_path).get('AppState', {}).get('installdir', None)
                    if not appmanifest_install_path or not os.path.isdir(os.path.join(fid_steamapps_path, 'common', appmanifest_install_path)):
                        continue

                app = SteamApp()
                app.app_id = int(appid)
                app.libraryfolder_id = fid
                app.libraryfolder_path = fid_path
                app.anticheat_runtimes = { RuntimeType.EAC: False, RuntimeType.BATTLEYE: False }  # Have to initialize as False here for some reason...
                if ct := c.get(appid):
                    app.compat_tool = ct.get('name')
                apps.append(app)
        apps = update_steamapp_info(steam_config_folder, apps)
        apps = update_steamapp_awacystatus(apps)
    except Exception as e:
        print('Error (get_steam_app_list): Could not get a list of all Steam apps:', e)
    else:
        if not no_shortcuts:
            apps.extend(get_steam_shortcuts_list(steam_config_folder, c))

    _cached_app_list = apps
    return apps


def get_steam_shortcuts_list(steam_config_folder: str, compat_tools: dict=None) -> list[SteamApp]:
    """
    Returns a list of Steam shortcut apps (Non-Steam games added to the library) and the compatibility tool they are using
    steam_config_folder = e.g. '~/.steam/root/config'
    compat_tools (optional): dict, mapping the compat tools from config.vdf. Will be loaded from steam_config_folder if not specified
    Return Type: list[SteamApp]
    """
    users_folder = os.path.realpath(os.path.join(os.path.expanduser(steam_config_folder), os.pardir, 'userdata'))
    config_vdf_file = os.path.join(os.path.expanduser(steam_config_folder), 'config.vdf')

    apps = []

    try:
        if not compat_tools:
            compat_tools = get_steam_vdf_compat_tool_mapping(vdf_safe_load(config_vdf_file))

        for userf in os.listdir(users_folder):
            user_directory = os.path.join(users_folder, userf)
            if not os.path.isdir(user_directory):
                continue

            shortcuts_file = os.path.join(user_directory,'config/shortcuts.vdf')
            if not os.path.exists(shortcuts_file):
                continue
        
            shortcuts_vdf = vdf.binary_load(open(shortcuts_file,'rb'))
            if 'shortcuts' not in shortcuts_vdf:
                continue

            for sid,svalue in shortcuts_vdf.get('shortcuts').items():
                app = SteamApp()
                appid = svalue.get('appid')
                if appid < 0:
                    appid = appid +(1 << 32) #convert to unsigned
                
                app.app_id = appid
                app.shortcut_id = sid
                app.shortcut_startdir = svalue.get('StartDir')
                app.shortcut_exe = svalue.get('Exe')
                app.shortcut_icon = svalue.get('icon')
                app.shortcut_user = userf
                app.app_type = 'game'
                app.game_name = svalue.get('AppName') or svalue.get('appname')
                if ct := compat_tools.get(str(appid)):
                    app.compat_tool = ct.get('name')
                apps.append(app)
    except Exception as e:
        print('Error (get_steam_shortcuts_list): Could not get a list of Steam shortcut apps:', e)
    
    return apps


def get_steam_game_list(steam_config_folder: str, compat_tool: BasicCompatTool | None=None, cached=False) -> list[SteamApp]:
    """
    Returns a list of installed Steam games and which compatibility tools they are using.
    Specify compat_tool to only return games using the specified tool.
    Return Type: list[SteamApp]
    """
    apps = get_steam_app_list(steam_config_folder, cached=cached)

    return [app for app in apps if app.app_type == 'game' and (compat_tool is None or app.compat_tool == compat_tool.get_internal_name() or ctool_is_runtime_for_app(app, compat_tool))]


def get_steam_ct_game_map(steam_config_folder: str, compat_tools: list[BasicCompatTool], cached=False) -> dict[BasicCompatTool, list[SteamApp]]:
    """
    Returns a dict that maps a list of Steam games to each compatibility given in the compat_tools parameter.
    Steam games without a selected compatibility tool are not included.
    Informal Example: { GE-Proton7-43: [GTA V, Cyberpunk 2077], SteamTinkerLaunch: [Vecter, Terraria] }
    Return Type: dict[BasicCompatTool, list[SteamApp]]
    """
    ct_game_map = {}

    apps = get_steam_app_list(steam_config_folder, cached=cached)

    ct_name_object_map = {ct.get_internal_name(): ct for ct in compat_tools}

    for app in apps:
        if app.app_type == 'game' and app.compat_tool in ct_name_object_map:
            ct_game_map.setdefault(ct_name_object_map.get(app.compat_tool), []).append(app)

    return ct_game_map