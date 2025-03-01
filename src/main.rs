#![allow(non_snake_case)]
use tray_icon_win::{menu::{ Menu, MenuEvent, MenuItem}, Icon, TrayIcon, TrayIconBuilder};
use tao::{
    event::Event,
    event_loop::{EventLoop,EventLoopBuilder, EventLoopProxy,ControlFlow},
    window::WindowBuilder,
};
use winreg::{enums::*,RegKey};
use tokio::{task,time::{sleep,Duration},sync::Mutex};
use std::sync::Arc;
use kira::{AudioManager,AudioManagerSettings,DefaultBackend,sound::static_sound::StaticSoundData};

struct Timer{
    Handle : Option<task::JoinHandle<()>>,
    Manager : Arc<Mutex<AudioManager>>,
    SoundData : Arc<StaticSoundData>,
    Run : bool
}

impl Timer {
    pub fn New(Path : String) -> Self{
        let Manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
        let SoundData = StaticSoundData::from_file(Path).expect("error");
        Timer{
            Handle:None,
            Manager : Arc::new(Mutex::new(Manager)),
            SoundData : Arc::new(SoundData),
            Run : false
        }
    }
    pub fn Run(&mut self){
        self.Run = true;
        let Manager = Arc::clone(&self.Manager);
        let SoundData = Arc::clone(&self.SoundData);
        self.Handle = Some(task::spawn(async move {
            let mut mgr = Manager.lock().await;
            loop {
                sleep(Duration::from_secs(60*25)).await;
                let _ = mgr.play((*SoundData).clone());
                sleep(Duration::from_secs(60*5)).await;
                let _ = mgr.play((*SoundData).clone());
            }
        }));
    }
    pub fn Stop(&mut self){
        self.Run = false;
        if let Some(Handle) = &self.Handle{
            Handle.abort();
        }
        self.Handle = None;
    }
}

enum TrayIconEvents {
    MenuEvnet(tray_icon_win::menu::MenuEvent)
}

fn IsDarkModeEnabled() -> bool{
    let HKCU :RegKey = RegKey::predef(HKEY_CURRENT_USER);
    let Personalize : RegKey = HKCU.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize").unwrap();
    let AppsUseLightTheme : u32 = Personalize.get_value("AppsUseLightTheme").unwrap();
    AppsUseLightTheme == 0
}
#[tokio::main]
async  fn main() {
    //Get Icon
    let Icons:Icon = if IsDarkModeEnabled() {Icon::from_path("public/WhiteIcon.ico", None).unwrap()}
    else{ Icon::from_path("public/BlackIcon.ico",None).unwrap() };
    //Menu Item Set
    let SetItem:MenuItem = MenuItem::new("Set", true, None);
    let ExitItem:MenuItem = MenuItem::new("Exit", true,None);
    //Menu
    let Menu:Menu = Menu::new();
    Menu.append(&SetItem).unwrap();
    Menu.append(&ExitItem).unwrap();
    //TrayIcon SetUp
    let TrayIcon: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(Menu))
        .with_icon(Icons)
        .build()
        .unwrap();
    // Event and Window Setup
    let Eventloop: EventLoop<TrayIconEvents> =   EventLoopBuilder::<TrayIconEvents>::with_user_event().build(); 
    let Windows = WindowBuilder::new().build(&Eventloop).unwrap();
    Windows.set_visible(false);
    //Proxy Event Setup
    let Proxy: EventLoopProxy<TrayIconEvents> = Eventloop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event|{
        Proxy.send_event(TrayIconEvents::MenuEvnet(event)).ok();
    }));
    //Timer setup
    let mut Timer : Timer = Timer::New("public/Alam.wav".to_owned()); 
    //EventLoop Run
    Eventloop.run(move |Event:Event<'_, TrayIconEvents>,_,ControlFlowState:&mut ControlFlow|{
        *ControlFlowState = ControlFlow::Wait;

        match Event {
            Event::UserEvent(TrayIconEvents::MenuEvnet(MenuEvnets))=>{
                match MenuEvnets {
                    MenuEvent { id } =>{
                        if id == SetItem.id(){
                            if Timer.Run == false{
                                Timer.Run();
                            }else {
                                Timer.Stop();
                            }
                        }
                        if id == ExitItem.id(){
                            *ControlFlowState = ControlFlow::Exit;
                        }
                    }
                }
            }
            _=> (),
        }
    });
}
