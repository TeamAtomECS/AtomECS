
use specs::{
	VecStorage,Component, Entities, Join, LazyUpdate, ReadStorage, System, WriteStorage,
};
use std::string::String;
use crate::atom::{Position,Atom};
extern crate gnuplot;
use gnuplot::*;
pub struct PositionRecord{
    pub x:Vec<f64>,
    pub y:Vec<f64>,
    pub z:Vec<f64>,
}
impl Component for PositionRecord{
    type Storage = VecStorage<Self>;
}

pub struct RecordPositionSystem;

impl <'a> System <'a> for RecordPositionSystem{
    type SystemData = (WriteStorage<'a,PositionRecord>,ReadStorage<'a,Position>);
    fn run (&mut self, (mut record,position): Self::SystemData){
        for (mut record,position) in (&mut record,&position).join(){
            //println!("recorded");
            record.x.push(position.pos[0]);
            record.y.push(position.pos[1]);
            record.z.push(position.pos[2]);
        }
    }
}
pub struct PlotSystem;

impl <'a> System <'a> for PlotSystem{
    type SystemData = (ReadStorage<'a,PositionRecord>,ReadStorage<'a,Atom>);
    fn run (&mut self, (record,atom): Self::SystemData){
        let mut fg = Figure::new();
        fg.axes3d()
	            .set_title("A plot", &[])
	            .set_x_label("x direction", &[])
	            .set_y_label("y direction", &[])
                .set_z_label("z direction", &[]);
                let mut n = 1;
        for (record,atom) in (&record,&atom).join(){
            n = n + 1;
            //println!("what{:?}",record.x);
            fg.axes3d()
	            .set_title("A plot", &[])
	            .lines(
		            &record.x,
		            &record.y,
                    &record.z,
	            	&[Caption(&n.to_string())],
	        );
            
        }
        fg.show();
    }
}

pub fn test(){
let mut fg = Figure::new();
fg.axes3d()
	.set_title("A plot", &[])
	.set_x_label("x", &[])
	.set_y_label("y", &[])
    .set_z_label("z", &[])
	.lines(
		&[-3., -2., -1., 0., 1., 2., 3.],
		&[9., 4., 1., 0., 1., 4., 9.],
        &[1.,1.,1.,1.,1.,1.,1.],
		&[Caption("Parabola")],
	);
fg.show();
}