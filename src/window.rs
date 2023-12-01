use sdl2::{EventPump, VideoSubsystem};

pub struct SDLWindow
{
    windowed : bool,
    windowed_width : u32,
    windowed_height : u32,
    window : Box<sdl2::video::Window>,
    pub event_pump: EventPump,
}

impl SDLWindow
{
    pub fn new(width : u32, height : u32, video_subsystem : VideoSubsystem, event_pump: EventPump) -> SDLWindow
    {
        let window = video_subsystem
            .window("Star Scouts Test Build", width, height)
            .position_centered()
            //.fullscreen_desktop()
            .build()
            .map_err(|e| e.to_string()).unwrap();
        return SDLWindow
        {
            windowed: false,
            windowed_width: width,
            windowed_height: height,
            window: Box::new(window),
            event_pump: event_pump,
        };
    }

    pub fn show(&mut self)
    {
        self.window.show();
    }

    pub fn get_window(&self) -> &sdl2::video::Window
    {
        return self.window.as_ref();
    }

}