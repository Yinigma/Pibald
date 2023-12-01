
#[cfg(test)]

#[test]
fn test_setup()
{
    /*use std::{ f32::consts::PI, rc::Rc, time::Duration, ops::Add, str::FromStr };

    use glam::{Vec3, Mat4, Vec4, Quat};
    use sdl2::{event::Event, keyboard::Keycode};

    use crate::{window::SDLWindow, renderer::{render_state::{render_state::RenderState, view::CameraDescriptor, light::{PointLightDescriptor}, animation::{AnimationState, AnimationStateBuilder, MixerBuilder, SimpleMixerBuilder, PlaybackType}}, gpu::{renderer::Renderer, gpu_store::GPUStore}, data::InMemoryModelRepository}};
    
    //SDL/wgpu setup
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    sdl_context.mouse().set_relative_mouse_mode(true);
    let event_pump = sdl_context.event_pump().unwrap();
    let mut window = SDLWindow::new(640, 480, video_subsystem, event_pump);

    let instance = wgpu::Instance::new
    (
        wgpu::InstanceDescriptor 
        {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        }
    );

    let surface = unsafe { instance.create_surface(&window.get_window()) }.unwrap();

    let adapter = pollster::block_on
    (
        instance.request_adapter
        (
            &wgpu::RequestAdapterOptions 
            {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )
    ).unwrap();

    let (device, queue) = pollster::block_on
    (
        adapter.request_device
        (
            &wgpu::DeviceDescriptor 
            {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        )
    ).unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())            
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration 
    {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: 640,
        height: 480,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    //get a test model
    let mut repo = InMemoryModelRepository::new();
    repo.load_static_model("axes".to_string(), "C:\\Users\\fobja\\3D Objects\\export_test\\axes.pibm".to_string());
    repo.load_animated_model("rho_anim".to_string(), "C:\\Users\\fobja\\3D Objects\\export_test\\rho.pibm".to_string());
    repo.load_armature("rho_armature".to_string(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\rho.pibs").ok().unwrap());
    repo.load_animation("idle".to_string(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\clip_idle.piba").ok().unwrap(), &"rho_armature".to_string());
    repo.load_animation("walk".to_string(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\clip_walk.piba").ok().unwrap(), &"rho_armature".to_string());
    let anim_state = AnimationStateBuilder::create_builder
    (
        repo.get_armature(&"rho_armature".to_string()).unwrap(), 
        vec!
        [
            MixerBuilder::Simple(SimpleMixerBuilder::new("idle", repo.get_animation(&"idle".to_string()).unwrap()).playback_type(PlaybackType::Looping)),
            MixerBuilder::Simple(SimpleMixerBuilder::new("walk", repo.get_animation(&"walk".to_string()).unwrap()).playback_type(PlaybackType::Looping)),
        ]
    ).base_starting_animation("idle").build();
    let anim_model = repo.get_animated_model(&"rho_anim".to_string()).unwrap();
    let model = repo.get_static_model(&"axes".to_string()).unwrap();
    let mut gpu_store = GPUStore::new();
    gpu_store.load_static_model(model, &device);
    gpu_store.load_skinned_model(anim_model, &device);
    println!("{}", anim_model.armature_id);
    //spin up test scene
    let mut state = RenderState::new();
    let group_id = state.add_group();
    let cam_id = state.add_camera
    (
        group_id, 
        CameraDescriptor
        { 
            loc: Vec3::new(0.0, 0.0, 0.0), 
            forward: Vec3::Z, 
            up: Vec3::Y,
            fov: 70.0 * PI / 180.0, 
            aspect: 4.0/3.0, 
            z_near: 0.01,
            z_far: 10000.0,
        }
    ).unwrap();
    let mut test_tf = Mat4::from_translation(Vec3::new(0.0, -3.0, 5.0));

    /*let mut arma_ids = vec![];
    let armature = repo.get_armature(&"rho_armature".to_string()).unwrap();
    {
        let mut arma_tfs = vec![];
        for i in 0..armature.bones.len()
        {
            let arma_parent = match armature.bones[i].parent 
            {
                Some(parent) => arma_tfs[parent],
                None => Mat4::IDENTITY,
            };
            arma_tfs.push(arma_parent * armature.bones[i].local_tf);
            arma_ids.push(state.add_static_model(group_id, model, arma_tfs[i] * Mat4::from_scale(Vec3::ONE * 0.2)).unwrap());
        }
    }*/

    let anim_mod = state.add_animated_model(group_id, anim_model, test_tf, anim_state).unwrap();
    
    let light = state.add_point_light(group_id, PointLightDescriptor::new()).unwrap();
    let light2 = state.add_point_light(group_id, PointLightDescriptor::new()).unwrap();
    state.get_group_mut(group_id).unwrap().get_point_light_mut(light).unwrap().light.location = Vec3::new(-1.5, 1.5, 4.0);
    state.get_group_mut(group_id).unwrap().get_point_light_mut(light2).unwrap().light.location = Vec3::new(3.5, 3.5, 2.0);
    let mut renderer = Renderer::new(&device, &config);
    renderer.attach_camera(cam_id, 640, 480, &device);
    let mut forward = false;
    let mut back = false;
    let mut left = false;
    let mut right = false;

    'running: loop
    {
        let output = surface.get_current_texture().unwrap();
        let target = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        renderer.set_output_view(cam_id, target);
        for event in window.event_pump.poll_iter() 
        {
            let camera = state.get_group_mut(group_id).unwrap().get_camera_mut(cam_id).unwrap();
            match event 
            {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => 
                {
                    break 'running
                },
                Event::KeyDown {keycode: Some(Keycode::A), .. } =>
                {
                    left = true;
                },
                Event::KeyUp {keycode: Some(Keycode::A), .. } =>
                {
                    left = false;
                },
                Event::KeyDown {keycode: Some(Keycode::D), .. } =>
                {
                    right = true;
                },
                Event::KeyUp {keycode: Some(Keycode::D), .. } =>
                {
                    right = false;
                },
                Event::KeyDown {keycode: Some(Keycode::W), .. } =>
                {
                    forward = true;
                },
                Event::KeyUp {keycode: Some(Keycode::W), .. } =>
                {
                    forward = false;
                },
                Event::KeyDown {keycode: Some(Keycode::S), .. } =>
                {
                    back = true;
                },
                Event::KeyUp {keycode: Some(Keycode::S), .. } =>
                {
                    back = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Space), .. } =>
                {
                    camera.translate(0.1 * camera.up());
                },
                Event::KeyDown {keycode: Some(Keycode::LCtrl), .. } =>
                {
                    camera.translate(-0.1 * camera.up());
                },
                Event::KeyDown { keycode: Some(Keycode::E), .. } =>
                {
                    state.get_group_mut(group_id).unwrap().get_animated_model_mut(anim_mod).unwrap().anim_state_mut().base_layer.queue_clip_mixer("walk", Duration::from_millis(300));
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } =>
                {
                    state.get_group_mut(group_id).unwrap().get_animated_model_mut(anim_mod).unwrap().anim_state_mut().base_layer.queue_clip_mixer("idle", Duration::from_millis(300));
                }
                Event::MouseMotion { xrel, yrel, .. } =>
                {
                    camera.pitch((-yrel as f32) / ( 480.0 ));
                    camera.yaw((xrel as f32) / ( 640.0 ));
                },
                _ => {}
            }
        }
        {
            let camera = state.get_group_mut(group_id).unwrap().get_camera_mut(cam_id).unwrap();
            if forward
            {
                camera.translate(2.0 / 60.0 * camera.forward());
            }
            if back
            {
                camera.translate(-2.0 / 60.0 * camera.forward());
            }
            if right
            {
                camera.translate(2.0 / 60.0 * camera.right());
            }
            if left
            {
                camera.translate(-2.0 / 60.0 * camera.right());
            }
        }
        
        test_tf = test_tf * Mat4::from_quat(Quat::from_axis_angle(Vec3::Y, 0.05));
        //state.get_group_mut(group_id).unwrap().get_static_model_mut(stat_mod).unwrap().set_transform(test_tf);
        state.get_group_mut(group_id).unwrap().get_animated_model_mut(anim_mod).unwrap().update(1.0/60.0);
        renderer.push_buffer_updates(&state, &device, &queue);
        state.clear_dirty_state();
        renderer.render(&state, &gpu_store, &device, &queue);
        output.present();
        // The rest of the game loop goes here...
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }*/
}



