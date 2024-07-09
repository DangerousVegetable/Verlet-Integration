use std::borrow::BorrowMut;

use crate::particle::Particle;

use glam::Vec2;
use itertools::Itertools;
#[derive(Clone)]
pub struct Simulation {
    pub constraint: Constraint,
    pub cell_size: f32, 
    pub grid: Grid<usize>,
}

impl Simulation {
    pub fn new(constraint: Constraint, cell_size: f32) -> Self {
        let bounds = constraint.bounds();
        let width: usize = ((bounds.1.x - bounds.0.x)/cell_size) as usize + 1;
        let height: usize = ((bounds.1.y - bounds.0.y)/cell_size) as usize + 1;

        Self {
            constraint,
            cell_size,
            grid: Grid::new(width, height)
        }
    }

    fn populate_grid(&mut self, particles: &mut [Particle]) {
        self.grid.clear();
        for (i, particle) in particles.iter().enumerate() {
            let p = self.get_cell(particle.pos);
            self.grid.push(p, i);
        }
    }

    fn get_cell(&self, pos: Vec2) -> (usize, usize) {
        let bounds = self.constraint.bounds().0;
        (((pos.x - bounds.x)/self.cell_size).max(0.) as usize, 
        ((pos.y - bounds.y)/self.cell_size).max(0.) as usize)
    }

    pub fn solve(&mut self, particles: &mut [Particle], dt: f32) {
        // populate the grid with indexes of particles
        self.populate_grid(particles); // TODO: for some reason it's slow in debug mode
        
        for p in particles.borrow_mut() {
            p.apply_gravity();
        }

        //let mut max = 0;
        //let mut sum = 0;
        let mut adj_buffer = Vec::new();
        for i in 0..particles.len() {
            let c = self.get_cell(particles[i].pos);
            adj_buffer.clear();
            self.grid.adjacent(c, &mut adj_buffer);
            for &j in &adj_buffer {
                let (i,j) = (std::cmp::min(i,j), std::cmp::max(i,j));
                if i == j {continue}
                let (head, tail) = particles.split_at_mut(i + 1); // such a hacky solution (but they say it's okay)
                Simulation::resolve_collision(&mut head[i], &mut tail[j - i - 1]);
            }
        }

        //println!("{} - {}", sum/particles.len(), max);

        for p in particles.borrow_mut() {
            p.update(dt);
        }

        for p in particles.borrow_mut() {
            p.apply_constraint(self.constraint);
        }
    }

    pub fn resolve_collision(p1: &mut Particle, p2: &mut Particle) {
        let mut v = p1.pos - p2.pos;
        if v.length() < p1.radius + p2.radius {
            let overlap = (p1.radius + p2.radius - v.length())/2.;
            v = v.normalize()*overlap;
            p1.set_position(p1.pos + v, true);
            p2.set_position(p2.pos - v, true);
        }
    }
}


#[derive(Clone)]
pub struct Grid<T> 
where T: Clone + Copy,
{
    grid: Vec<Vec<Vec<T>>>
}

impl<T> Grid<T> 
where T: Clone + Copy,
{
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![vec![Vec::new(); height]; width]
        }
    }

    pub fn clear(&mut self) {
        for col in self.grid.iter_mut() {
            for cell in col.iter_mut() {
                cell.clear();
            }
        }
    }

    pub fn push(&mut self, (i,j): (usize, usize), value: T) {
        self.grid[i][j].push(value);
    }

    pub fn at(&self, (i, j): (usize, usize)) -> &Vec<T> { 
        &self.grid[i][j]
    }

    pub fn adjacent(&self, (i,j): (usize, usize), buffer: &mut Vec<T>) {
        let (i,j) = (i as isize, j as isize);

        let indexes = [(i-1, j-1), (i-1, j), (i-1, j+1),
        (i, j-1), (i, j), (i, j+1),
        (i+1, j-1), (i+1, j), (i+1, j+1)];

        for (i, j) in indexes {
            if 0 <= i && 0 <= j &&
            i < self.grid.len() as isize && j < self.grid[0].len() as isize {
                let p = (i as usize, j as usize);
                buffer.extend(self.at(p));
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

