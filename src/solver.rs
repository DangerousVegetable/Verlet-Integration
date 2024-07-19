use std::{f32::consts::PI, ops::{Index, IndexMut}};

use crate::particle::{Particle, METAL, SAND};

use glam::Vec2;
use rand::Rng;

pub const MAX: u32 = 100000;
pub const PARTICLE_SIZE: f32 = 0.1;

pub type Connection = (usize, usize, Link);
#[derive(Clone)]
pub struct Simulation {

    pub constraint: Constraint,
    pub particles: Vec<Particle>,
    pub connections: Vec<Connection>,
    pub cell_size: f32, 
    pub grid: Grid<usize>,
}

impl Simulation {
    pub fn new(constraint: Constraint, cell_size: f32, particles: &[Particle], connections: &[Connection]) -> Self {
        let bounds = constraint.bounds();
        let width: usize = ((bounds.1.x - bounds.0.x)/cell_size) as usize + 1;
        let height: usize = ((bounds.1.y - bounds.0.y)/cell_size) as usize + 1;

        Self {
            constraint,
            particles: Vec::from(particles),
            connections: Vec::from(connections),
            cell_size,
            grid: Grid::new(width, height)
        }
    }

    fn populate_grid(&mut self) {
        self.grid.clear();
        for (i, particle) in self.particles.iter().enumerate() {
            let p = self.get_cell(particle.pos);
            self.grid.push(p, i);
        }
    }

    fn get_cell(&self, pos: Vec2) -> (usize, usize) {
        let bounds = self.constraint.bounds().0;
        (((pos.x - bounds.x)/self.cell_size).max(0.) as usize, 
        ((pos.y - bounds.y)/self.cell_size).max(0.) as usize)
    }

    pub fn solve(&mut self, dt: f32) {
        // populate the grid with indexes of particles
        self.populate_grid(); // TODO: for some reason it's slow in debug mode
        
        self.apply_gravity();

        self.resolve_collisions();

        self.resolve_connections();

        self.update_particles(dt);

        self.apply_constraint();
    }

    fn apply_gravity(&mut self) {
        for p in self.particles.iter_mut() {
            p.apply_gravity();
        }
    }

    fn resolve_collisions(&mut self) {
        let mut adj_buffer = Vec::new();
        for i in 0..self.particles.len() {
            let c = self.get_cell(self.particles[i].pos);
            adj_buffer.clear();
            self.grid.adjacent(c, &mut adj_buffer);
            for &j in &adj_buffer {
                let (i,j) = (std::cmp::min(i,j), std::cmp::max(i,j));
                if i == j {continue}
                let (head, tail) = self.particles.split_at_mut(i + 1); // such a hacky solution (but they say it's okay)
                Simulation::resolve_collision(&mut head[i], &mut tail[j - i - 1]);
            }
        }
    }

    fn resolve_connections(&mut self) {
        self.connections
        .retain( |&(i, j, _)| {
            i < self.particles.len() && j < self.particles.len()
        });
        for &(i, j, link) in self.connections.iter() {
            let (i, j) = (std::cmp::min(i, j), std::cmp::max(i, j));
            let (head, tail) = self.particles.split_at_mut(i + 1);
            Simulation::resolve_connection(&mut head[i], &mut tail[j - i - 1], link);
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.update(dt);
        }
    }

    fn apply_constraint(&mut self) {
        for p in self.particles.iter_mut() {
            p.apply_constraint(self.constraint);
        }
    }

    pub fn resolve_collision(p1: &mut Particle, p2: &mut Particle) {
        let mut v = p1.pos - p2.pos;
        if v.length() < p1.radius + p2.radius {
            let overlap = (p1.radius + p2.radius - v.length());
            let c1 = p2.mass/(p1.mass + p2.mass);
            let c2 = p1.mass/(p1.mass + p2.mass);
            v = v.normalize()*overlap;
            p1.set_position(p1.pos + v*c1, true);
            p2.set_position(p2.pos - v*c2, true);
        }
    }

    pub fn resolve_connection(p1: &mut Particle, p2: &mut Particle, link: Link) {
        match link {
            Link::Force(force) => {
                let v = (p2.pos - p1.pos).normalize_or_zero();
                p1.accelerate(v*force);
                p2.accelerate(-v*force);
            },
            Link::Rigid(length) => {
                let mut v = p1.pos - p2.pos;
                let overlap = (length-v.length())/2.;
                v = overlap*v.normalize();
                p1.set_position(p1.pos+v, true);
                p2.set_position(p2.pos-v, true);
            }
        }
    }

    pub fn change_number(&mut self, number: usize) {
        if number < self.particles.len() {
            self.particles.truncate(number);
        }
        else {
            while self.particles.len() < number {
                let left = number-self.particles.len();
                //if left >= 20 {
                //    self.add_ring(0.3, 20);
                //}
                //else if left >= 4 {
                    //    self.add_square(1.0);
                //}
                //else if left >= 3 {
                    //    self.add_triangle(1.0);
                //}
                //else {self.add_particle(SAND.place(self.rnd_origin()));}
                
                if self.particles.len() % 10 == 0 {
                    self.add_particle(METAL.place(self.rnd_origin()))
                }
                else {
                    self.add_particle(SAND.place(self.rnd_origin()))
                }
            }
        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    pub fn add_rib(&mut self, i: usize, j: usize, length: f32) {
        self.connections.push((i, j, Link::Rigid(length)))
    }

    pub fn add_spring(&mut self, i: usize, j: usize, force: f32) {
        self.connections.push((i, j, Link::Force(force)))
    }

    pub fn add_triangle(&mut self, length: f32) {
        self.add_ring(length, 3);
    }

    pub fn add_square(&mut self, length: f32) {
        let ind = self.particles.len();
        self.add_ring(length, 4);
        self.add_rib(ind, ind+2, length*f32::sqrt(2.));
        self.add_rib(ind+1, ind+3, length*f32::sqrt(2.));
    }

    pub fn add_ring(&mut self, length: f32, number: usize) {
        let angle = PI/(number as f32);
        let radius = 0.5*length/f32::atan(angle);
        let center = rnd_in_bounds(self.constraint.bounds(), radius);
        let ind = self.particles.len();
        for i in 0..number {
            let alpha = 2.*PI*(i as f32)/(number as f32);
            let pos = center + glam::vec2(f32::cos(alpha), f32::sin(alpha))*radius;
            self.add_particle(SAND.place(pos));
            self.add_rib(ind+i, ind+((i+1) % number), length);
        }
    }

    fn rnd_origin(&self) -> Vec2 {
        let bounds = self.constraint.bounds();
        rnd_in_bounds(bounds, 2.*PARTICLE_SIZE)
    }
}

pub fn rnd_in_bounds(bounds: (Vec2, Vec2), margin: f32) -> Vec2 {
    Vec2::new(
        rand::thread_rng().gen_range(bounds.0.x+margin ..bounds.1.x-margin),
        rand::thread_rng().gen_range(bounds.0.y+margin ..bounds.1.y-margin),
    )
}

#[derive(Clone, Copy)]
pub enum Link {
    Force(f32), // force
    Rigid(f32), // constant length
}

#[derive(Clone)]
pub struct Grid<T> 
where T: Clone + Copy,
{
    pub width: usize, 
    pub height: usize,
    grid: Vec<Vec<T>>
}

impl<T> Index<(usize, usize)> for Grid<T>
where T: Clone + Copy 
{
    type Output = Vec<T>;
    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        let ind = i*self.width + j;
        &self.grid[ind]
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T>
where T: Clone + Copy
{
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        let ind = i*self.width + j;
        &mut self.grid[ind]
    }
}

impl<T> Grid<T> 
where T: Clone + Copy,
{
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![Vec::new(); width*height]
        }
    }

    pub fn clear(&mut self) {
        for cell in self.grid.iter_mut() {
            cell.clear()
        }
    }

    pub fn push(&mut self, ind: (usize, usize), value: T) {
        self[ind].push(value);
    }

    pub fn adjacent(&self, (i,j): (usize, usize), buffer: &mut Vec<T>) {
        let (i,j) = (i as isize, j as isize);

        let indexes = [(i-1, j-1), (i-1, j), (i-1, j+1),
        (i, j-1), (i, j), (i, j+1),
        (i+1, j-1), (i+1, j), (i+1, j+1)];

        for (i, j) in indexes {
            if 0 <= i && 0 <= j &&
            i < self.width as isize && j < self.height as isize {
                let p = (i as usize, j as usize);
                buffer.extend(&self[p]);
            }
        }
    }
}


#[derive(Clone, Copy)]
pub enum Constraint {
    Box (Vec2, Vec2), // Rectangle, bottom-left and top-right corners 
    #[deprecated]
    Cup (Vec2, Vec2) // U-shape, bottom-left and top-right corners
}

impl Constraint {
    pub const fn bounds(&self) -> (Vec2, Vec2) {
        match self {
            &Constraint::Box(bl, tr) => (bl, tr),
            &Constraint::Cup(bl, tr) => (bl, tr)
        }
    }
}

