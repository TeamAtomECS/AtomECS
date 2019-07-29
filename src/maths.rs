pub mod Maths{
	use crate::constant::BOLTZCONST;
	use crate::constant::PI;
	use crate::constant::EXP;
	extern crate rand;
	use rand::Rng;
	pub fn array_addition(a:&[f64;3],b:&[f64;3]) -> [f64;3]{
		//addition of 3D array
		//checked
		let mut result = [0.0,0.0,0.0];
		for i in 0..3{
			result[i]=a[i]+b[i];
		}
		result
	}
	pub fn array_multiply(a:&[f64;3],b:f64) -> [f64;3]{
		//checked
		[a[0]*b,a[1]*b,a[2]*b]
	}
	pub fn dot_product(a:&[f64;3],b:&[f64;3]) -> f64{
		//dot product
		//checked
		a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
	}
	pub fn cross_product(a:&[f64;3],b:&[f64;3]) -> [f64;3]{
		//cross product
		//checked
		[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
	}
	pub fn modulus(a:&[f64;3]) -> f64{
		//checked
		(a[0].powf(2.0)+a[1].powf(2.0)+a[2].powf(2.0)).powf(0.5)
	}
	
	pub fn norm(a:&[f64;3]) -> [f64;3]{
		array_multiply(&a,1.0/modulus(&a))
	}
	pub fn gaussian_dis(std:f64,distance:f64)->f64{
		//checked
		1.0/(2.0*PI*std.powf(2.0))*EXP.powf(-distance.powf(2.0)/2.0/(std).powf(2.0))
	}
	pub fn maxwell_dis(_t:f64,_mass:f64,_velocity:f64) -> f64{
		(_mass/2.0/PI/BOLTZCONST/_t).powf(1.5)*EXP.powf(-_mass*_velocity.powf(2.0)/2.0/BOLTZCONST/_t)*4.0*PI*_velocity.powf(2.0)
	}
	
	pub fn maxwell_generate(_t:f64,_mass:f64) -> f64{
		// take about 20 times of the variance as range and do random uniform generation
		// use 1/1000 times of the real PDF so that the maxwell distribution is everywhere lower than the uniform one
		
		let range = 20.0 * (BOLTZCONST*_t/_mass).powf(0.5);
		let mut i = 0;
		loop{
			let mut rng = rand::thread_rng();
			i = i + 1;

			let result = rng.gen_range(0.0, range);
			let height = rng.gen_range(0.0, 1.0/range);
			if maxwell_dis(_t,_mass,result)>height*1000.0{
				return result
			}
		}
	}
	
	pub fn RK4(f:&Box<Fn(&[f64;3],&[f64;3])->[f64;3] +Send+Sync>,v0:&[f64;3],y0:&[f64;3], delta_t:f64) -> [[f64;3];2]{
		//checked
		let k1 = array_multiply(&f(&v0,&y0),delta_t);
		let l1 = array_multiply(&v0,delta_t);
		let y1 = array_addition(&y0,&array_multiply(&l1,0.5));
		let v1 = array_addition(&v0,&array_multiply(&k1,0.5));
		let k2 = array_multiply(&f(&v1,&y1),delta_t);
		let l2 = array_multiply(&v1,delta_t);
		
		let y2 = array_addition(&y0,&array_multiply(&l2,0.5));
		let v2 = array_addition(&v0,&array_multiply(&k2,0.5));
		let k3 = array_multiply(&f(&v2,&y2),delta_t);		
		let l3 = array_multiply(&v2,delta_t);
		
		let y3 = array_addition(&y0,&l3);
		let v3 = array_addition(&v0,&k3);
		let k4 = array_multiply(&f(&v3,&y3),delta_t);		
		let l4 = array_multiply(&v3,delta_t);
		

		[[y0[0]+l1[0]/6.0+l2[0]/3.0+l3[0]/3.0+l4[0]/6.0,y0[1]+l1[1]/6.0+l2[1]/3.0+l3[1]/3.0+l4[1]/6.0,y0[2]+l1[2]/6.0+l2[2]/3.0+l3[2]/3.0+l4[2]/6.0],[v0[0]+k1[0]/6.0+k2[0]/3.0+k3[0]/3.0+k4[0]/6.0,v0[1]+k1[1]/6.0+k2[1]/3.0+k3[1]/3.0+k4[1]/6.0,v0[2]+k1[2]/6.0+k2[2]/3.0+k3[2]/3.0+k4[2]/6.0]]
	
	}	
	
	pub fn jtheta(theta:f64)-> f64{
		//checked (against dongyang matlab code)
		// problem remained what is it? 
		let r = 0.2e-3;
		let L = 4.0e-3;
		let beta = 2.0*r/L;
		let q_theta = 1.0/beta*theta.tan();
		let r_theta = q_theta.acos() - q_theta*(1.0-q_theta.powf(2.0)).powf(0.5);
		let alpha = 1.0/2.0 - 1.0/(3.0*beta.powf(2.0))*(1.0-2.0*beta.powf(3.0) + (2.0*beta.powf(2.0) - 1.0)*(1.0+beta.powf(2.0)).powf(0.5))/((1.0+beta.powf(2.0)).powf(0.5) - beta.powf(2.0)*(1.0/beta).asinh());
		let j1_theta = alpha*theta.cos() + 2.0/PI*theta.cos()*((1.0-alpha)*r_theta + 2.0/(3.0*q_theta)*(1.0-2.0*alpha)*(1.0-(1.0-q_theta.powf(2.0)).powf(3.0/2.0)));
		let j2_theta = alpha*theta.cos() + 4.0/(3.0*PI*q_theta)*(1.0-2.0*alpha)*theta.cos();
		if q_theta < 1.0{
			j1_theta*2.0*PI*theta.sin()
		}
		else{
			j2_theta*2.0*PI*theta.sin()
		}
	}
	pub fn jtheta_gen()-> f64{
		//generate a random sample based on jtheta distribution
		// use 1/20 of the jetha function so that the uniform distribution is always above the jtheta
		let mut i = 0;
		loop{
			let mut rng = rand::thread_rng();
			i = i + 1;

			let result = rng.gen_range(0.0, PI/2.0);
			let height = rng.gen_range(0.0, 2.0/PI);
			if jtheta(result)>height*20.0{
				return result
			}
		}
	}
	pub fn random_direction()->[f64;3]{
		let mut rng = rand::thread_rng();
		let angle1 = rng.gen_range(0.0, PI);
		let angle2 = rng.gen_range(0.,2.*PI);
		let result = [angle1.cos(),angle1.sin()*angle2.sin(),angle1.sin()*angle2.cos()];
		//println!("{:?}",result);
		result
	}
	#[cfg(test)]
	mod tests {
    use super::*;

    #[test]
    fn array_algebra() {
		assert_eq!(array_addition(&[1.,1.,1.],&[10.,10.,10.]),[11.,11.,11.]);
		assert_eq!(array_multiply(&[1.,2.,3.],10.),[10.,20.,30.]);
		assert_eq!(dot_product(&[1.,2.,3.],&[3.,2.,1.]),10.);
		assert_eq!(cross_product(&[1.,2.,3.],&[3.,2.,1.]),[-4.,8.,-4.]);
		assert_eq!(modulus(&[1.,2.,2.]),3.);
		assert_eq!(norm(&[1.,2.,2.]),[1./3.,2./3.,2./3.]);
    }
		#[test]
		fn distribution_test(){
			assert!(jtheta(1.)>0.2174 && jtheta(1.)<0.2176,"jtheta ");
			
			assert!(maxwell_dis(300.,1e-25,100.)>0.000839 && maxwell_dis(300.,1e-25,100.)<0.000840);
		}
}
}