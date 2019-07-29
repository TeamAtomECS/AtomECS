extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,Component,VecStorage};
use crate::atom::Position;
use crate::maths::Maths;
pub struct MagSampler{
	pub mag_sampler:[f64;3]
}

impl Component for MagSampler{
	type Storage = VecStorage<Self>;
}

pub struct MagFieldQuogra{
	pub gradient:f64,
	pub centre:[f64;3],
}

impl Component for MagFieldQuogra{
	//this component represent a source of quodrapole field
	type Storage = VecStorage<Self>;
}

pub struct UpdateSampler;

impl <'a> System<'a> for UpdateSampler{
	// based on Magnetic field component presented, magnetic sampler will be added for every entity with a position
		type SystemData = (WriteStorage<'a,MagSampler>,
									ReadStorage<'a,Position>,
									ReadStorage<'a,MagFieldQuogra>,
									);
	fn run(&mut self,(mut _sampler,pos,_mag_gauss):Self::SystemData){
		
		for _mag_gauss in (&_mag_gauss).join(){
			
			for (pos,mut sampler) in (&pos,&mut _sampler).join(){

				let _gradient = _mag_gauss.gradient;
				let _centre = _mag_gauss.centre;
				let rela_pos = Maths::array_addition(&pos.pos,&Maths::array_multiply(&_centre,-1.));
				sampler.mag_sampler = Maths::array_multiply(&[-rela_pos[0],-rela_pos[1],2.0*rela_pos[2]],_gradient);
			}
		}
	}
}
