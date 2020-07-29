
extern crate nalgebra;
use nalgebra::{Vector3,MatrixArray,MatrixVec,Dynamic};

use crate::atom::AtomInfo;
extern crate specs;

use crate::integrator::Timestep;
use crate::laser::LaserSamplers;
use crate::force::Force;
use specs::{
    Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, VecStorage,
    WriteStorage,ReadExpect
};
use crate::laser::repump::*;
use crate::constant;
extern crate specs;
use crate::atom::{Atom, AtomInfo};
use crate::constant;
use crate::maths;

use crate::atom::Force;
use crate::constant::{HBAR, PI};
use crate::integrator::Timestep;
use crate::magnetic::MagneticFieldSampler;


use crate::atom::Force;
pub struct DensityMatrixOption;

pub struct DensityMatrix{
    pub DensityMatrix:Matrix<f32, Dynamic, Dynamic, MatrixArray<f32, Dynamic,Dynamic>>,
}

impl DensityMatrix {
    pub fn new(&self, dimension: i64) -> DensityMatrix {
        let mut new_matrix=MatrixMN::<f64,dimension,dimension>::repeat(0.0);
        new_matrix[(0,0)]=1.0;
        return new_matrix
    }
}

pub struct DensityMatrixInitSystem;

impl<'a> System<'a> for DensityMatrixInitSystem{
    type SystemData =(
        ReadStorage<'a,atom>,
        ReadStorage<'a,atominfo>,
        ReadStorage<'a,NewlyCreated>,
        WriteStorage<'a,DensityMatrix>,
        ReadExpect<'a,DensityMatrixOption>
        Read<'a, LazyUpdate>,
        Entities<'a>,
    )
    run(&mut self, (_atom,atominfo,_newlycreated, mut dmatrix, mat_opt, updater,ent): Self::SystemData) {
        let mut matrixoption =false;
        match mat_opt {
            None => (),
            Some(_rand) => {
                matrixoption = true;
            }
        }
        if matrixoption{
            for (_,_new,mut dmatrix,atominfo) in
            (_atom,_newlycreated,&mut dmatrix,&atominfo).join(){
                updater.insert(ent,DensityMatrix::new(atominfo.number_of_level));
            }
        }
    }
}

pub struct DensityMatrixEvolutionSystem;
impl<'a> System<'a> for DensityMatrixEvolutionSystem{
    type SystemData = (
    ReadStorage<'a, LaserSamplers>,
    ReadStorage<'a, Atom>,
    ReadStorage<'a, MagneticFieldSampler>,
    ReadExpect<'a, Timestep>,
    ReadStorage<'a, Dark>,
    WriteStorage<'a, DensityMatrix>,
    ReadExpect<'a,DensityMatrixOption>.
    );
    /// evolve the density matrix based on the external environment
    fn run(&mut self, (samplers, _atom,mag_sampler, timestep, _dark, mut dmatrix,mat_opt): Self::SystemData) {
        let mut matrixoption =false;
        match mat_opt {
            None => (),
            Some(_rand) => {
                matrixoption = true;
            }
        }
        if matrixoption{
            for (samplers,mag_sampler _, atom_info, (), dmatrix) in
            (&samplers,&mag_sampler &_atom, &atom_info, !&_dark, &mut dmatrix).join(){
                for i in range(dmatrix.nrows()){
                    for j in range(dmatrix.ncols()){
                        dmatrix[(i,j)] = dmatrix[(i,j)] + timestep.delta    ;
                    }
                }
            }
        }
    }
}

pub struct DensityMatrixForceCalculation; 

impl<'a> System<'a> for DensityMatrixForceSystem{
    type SystemData = (
    ReadStorage<'a, LaserSamplers>,
    ReadStorage<'a, Atom>,
    ReadStorage<'a, MagneticFieldSampler>,
    ReadExpect<'a, Timestep>,
    ReadStorage<'a, Dark>,
    ReadStorage<'a, DensityMatrix>,
    WriteStorage<'a,Force>,
    ReadExpect<'a,DensityMatrixOption>,
    );
    /// evolve the density matrix based on the external environment
    fn run(&mut self, (samplers, _atom,mag_sampler, timestep, _dark, mut dmatrix,mut force,mat_opt): Self::SystemData) {
        let mut matrixoption =false;
        match mat_opt {
            None => (),
            Some(_rand) => {
                matrixoption = true;
            }
        }
        if matrixoption{
            for (samplers,mag_sampler _, atom_info, (), dmatrix,mut force) in
            (&samplers,&mag_sampler &_atom, &atom_info, !&_dark, &mut dmatrix,&mut force).join(){
                force.force = 
            }
        }
    }
}