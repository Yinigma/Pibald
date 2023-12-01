use glam::Vec3;

use super::InMemoryModelRepository;

#[cfg(test)]

#[test]
fn test_file_read()
{
    /*use std::str::FromStr;

    let mut repo = InMemoryModelRepository::new();
    repo.load_static_model(String::from_str("rho").ok().unwrap(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\Cube.pibm").ok().unwrap());
    let model = &repo.static_models["rho"];
    for v in &model.vertices
    {
        println!("loc: <{}, {}, {}>", v.loc.x, v.loc.y, v.loc.z);
        println!("norm: <{}, {}, {}>\n", v.norm.x, v.norm.y, v.norm.z);
    }*/
}

#[test]
fn test_armature_read()
{
    /*use std::str::FromStr;
    let mut repo = InMemoryModelRepository::new();
    repo.load_armature("rho_armature".to_string(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\rho.pibs").ok().unwrap());
    let armature = &repo.armatures["rho_armature"];*/
}

#[test]
fn test_animation_read()
{
    /*use std::str::FromStr;
    let mut repo = InMemoryModelRepository::new();
    let arma_key = "rho_armature".to_string();
    repo.load_armature(arma_key.clone(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\rho.pibs").ok().unwrap());
    repo.load_animation(String::from_str("jump").ok().unwrap(), String::from_str("C:\\Users\\fobja\\3D Objects\\export_test\\clip_jump.piba").ok().unwrap(), &arma_key);*/
}