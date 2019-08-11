use crate::atom::{Atom, Position};
use specs::{Read,LazyUpdate,Component, Entities, Join, NullStorage, ReadStorage, System, HashMapStorage};
extern crate nalgebra;
use nalgebra::Vector3;
use crate::destructor::ToBeDestroyed;
pub struct Detector{
    radius:f64,
    thickness:f64,
    direction:Vector3<f64>,
}

impl Detector{
    pub fn if_detect(&self,pos:&Vector3<f64>) -> bool{
        let dir = pos.normalize();
        let dis_vertical = dir.dot(&pos);
        let dis_radial = (pos.norm_squared() - dis_vertical.powf(2.0)).powf(0.5);
        (dis_vertical > -0.5* self.thickness)&&(dis_vertical < 0.5* self.thickness)&&(dis_radial < self.radius)
    }
}

impl Component for Detector{
	type Storage = HashMapStorage<Self>;
}

pub struct DetectingAtomSystem;

impl <'a> System <'a> for DetectingAtomSystem{
    type SystemData = (ReadStorage<'a, Position>,
                        ReadStorage<'a, Detector>,
                        Entities<'a>,
                        ReadStorage<'a, Atom>,
                        Read<'a,LazyUpdate>);
    fn run(&mut self,(pos,detector,ent,_atom,lazy):Self::SystemData){
        for (detector_pos,detector) in (&pos,&detector).join(){
            for (atom_pos,_,ent) in (&pos,&_atom,&ent).join(){
                let rela_pos = atom_pos.pos - detector_pos.pos;
                if detector.if_detect(&rela_pos){
                    lazy.insert(ent,ToBeDestroyed);
                }
            }
        }
    }
}