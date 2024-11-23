use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use rayon::prelude::*;
use fastrand::*;

#[derive(
    Debug, Clone, Copy, Serialize, DeserializeDebug, 
    Eq, PartialEq, Ord, PartialOrd, Hash
)]
#[serde(untagged)]
enum Direction{
    /// West
    W,
    /// East
    E,
    /// North
    N,
    /// South
    S,
    /// Up
    U,
    /// Down
    D
}

impl Direction{
    fn difference(&self) -> (usize, isize){
        match self{
            Direction::W => (0, -1),
            Direction::E => (0, 1),
            Direction::N => (1, -1),
            Direction::S => (1, 1),
            Direction::U => (2, -1),
            Direction::D => (2, 1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Relation{
    Isotropic(HashSet<usize>),
    Anisotropic(HashMap<Direction, HashSet<usize>>)
}

impl Relation{
    fn get_variants(&self, direction: Direction) -> Some(Vec<usize>){
        match self{
            Relation::Isotropic(neighbours) => Some(neighbours),
            Relation::Anisotropic(neighbours) => neighbours.get(&direction)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub struct Relations(Vec<Relation>);

impl Relations{
    fn get_all_anisotropic(&self) -> Vec<(usize, Relation)>{
        self.0.iter().enumerate()
            .filter(|(_, r)| if let Relation::Anisotropic(_) = r {
                true
            }else{
                false
            })
            .collect::<Vec<_>>()
    }

    fn get_all_isotropic(&self) -> Vec<(usize, Relation)>{
        self.0.iter().enumerate()
            .filter(|(_, r)| if let Relation::Isotropic(_) = r {
                true
            }else{
                false
            })
            .collect::<Vec<_>>()
    }

    fn try_transform_to_anisotropic(&mut self){
        let anisotropic = self.get_all_anisotropic();
        let isotropic = self.get_all_isotropic();
        if anisotropic.is_empty(){
            return;
        }
        let mut directions = HashSet::<Direction>::new();
        for (_, r) in anisotropic{
            let Relation::Anisotropic(map) = r else{
                unreachable!()
            };
            directions = directions.intersection(map.keys().collect::<HashSet<_>>())
                .collect::<HashSet<_>>();
        }
        for (i, r) in isotropic{
            let Relation::Isotropic(variants) = r else{
                unreachable!()
            };
            let mut map = HashMap::new();
            for d in directions{
                map.insert(d, variants);
            }
            self.0[i] = Relation::Anisotropic(map);
        }
        directions
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct Point(isize, isize);

struct Wfc{
    edges: HashMap<Direction, Vec<usize>>,
    map: Vec<Vec<Vec<usize>>>,
    front: Vec<Point>,
    relations: Relations
}

impl Wfc{
    fn new() -> Self{
        Self { 
            edges: HashMap::new(), 
            map: vec![], 
            front: vec![],
            relations: Relations(vec![])
        }
    }

    fn size(self, size: (usize, usize)) -> Self{
        self.map = vec![vec![vec![];size.0];size.1];
    }

    fn edges(self, edges: HashMap<Direction, Vec<usize>>) -> Self{
        self.edges = edges;
    }

    fn front(self, front: Vec<(Point, usize)>) -> Self{
        self.front = front;
    }

    fn relations(self, mut relations: Relations) -> Self{
        relations.try_transform_to_anisotropic();
        self.relations = relations; 
    }

    fn get_edges(&self) -> HashMap<Direction, Vec<usize>>{
        let mut edges = HashMap::new();
        let height = self.map.len();
        let width = self.map[0].len();
        let last_h = height - 1;
        let last_w = width - 1;
        for edge in self.edges.keys(){
            let (dim, diff) = edge.difference(); 
            let line = match (dim, diff){
                (0, -1) => (0..width).map(|i| self.map[0][i][0]),
                (1, -1) => (0..height).map(|i| self.map[i][0][0]),
                (0, 1) => (0..width).map(|i| self.map[last_h][i][0]),
                (1, 1) => (0..height).map(|i| self.map[i][last_w][0]),
                _ => unreachable!()
            }.collect::<Vec<_>>();
            edges.insert(edge, line);
        }
        self
    }

    fn calculate_variants(&self, points: Vec<Point>) -> Vec<(Point, Vec<usize>)>{
        let relations = self.relations.clone();
        let directions = self.edges.keys().cloned().collect::<Vec<_>>();
        let map = Arc::new(self.map);
        points.into_par_iter().map_init(
        || (map, directions, relations).clone(),
        |(map, directions, relations), p|{
            let new_variants = propagate_cell(&map, &directions, &relations, p);
            (p, new_variants)
        }).collect()
    }

    fn get_with_min_entrophy(&self, points: Vec<Point>) -> Vec<Point>{
        let map = Arc::new(self.map);
        let iter = points.into_par_iter().map_init(|| map.clone(),
        |m, p|{
            (p, m[p.1][p.0].len())
        }).filter(|m, (p, v)| v > 1);
        let min_entropy = iter.map(|m, (_, v)| v).min().unwrap();
        iter.filter(|m, (p, v)| v <= min_entropy)
            .map(|m, (p, _)| p).collect::<Vec<_>>()
    }

    fn collapse(&mut self){
        let front = self.front;
        let map = Arc::new(self.map);
        let collapsed = front.into_par_iter().map_init(|| (map.clone(), SmallRng::from_rng(rand::thread_rng())),
         |(m, r), p|{
            (p, m[p.1][p.0].choose(&mut r))
        }).collect::<Vec<_>>();
        for (p, v) in collapsed{
            self.map[p.1][p.0] = vec![v];
        }
    }

    fn propagate(&mut self){
        if self.map.is_empty() || self.map[0].is_empty(){
            return;
        }
        let front = self.front;
        let mut diffs = vec![];
        for edge in self.edges.keys(){
            diff.push(edge.difference());
        }
        // 1. Calculate new front. 
        let mut new_front = get_neighbours(front);
        new_front = get_with_min_entrophy(new_front);
        // 2. Find neighbours of new front.
        let new_front_surroundings = get_neighbours_map(new_front);
        // 3. Get their variants.
        // 4. Get interseption of neighbour variants.
        let new_variants = self.calculate_variants(new_front_surroundings);
        // 5. Set result as tile variants.
        for (p, variants) in new_variants{
            self.map[p.1][p.0] = variants;
        }
        self.front = new_front;
    }

    fn calculate(&mut self) -> Result<Vec<Vec<usize>>, &str>{
        while !self.front.is_empty(){
            self.collapse();
            self.propagate();
        }
        let is_not_completed = self.map.into_par_iter()
            .any(|row| {
                row.iter().any(|v| v.len() > 1)
            });
        if is_not_completed{
            return Err("Map is not complete even when front is empty");
        }
        Ok(self.map.into_par_iter().map(|row|{
            row.iter().flatten().collect()
        }).collect())
    }
}

fn propagate_cell(
    map: &Arc<Vec<Vec<Vec<usize>>>>, 
    directions: &Vec<Direction>,
    relations: &Relations,
    point: Point
) -> Vec<usize>{
    if relations.get_all_anisotropic().is_empty(){
        calculate_isotropic_variants(map, directions, relations, point)
    }else{
        calculate_anisotropic_variants(map, directions, relations, point)
    }
}

fn get_neighbour_variants(
    map: &Arc<Vec<Vec<Vec<usize>>>>,
    point: Point,
    diff: (usize, isize)
) -> Vec<usize>{
    let (dim, diff) = diff;
    match (dim, diff){
        (0, -1) => map[point.1][point.0 - 1],
        (1, -1) => map[point.1 - 1][point.0],
        (0, 1) => map[point.1][point.0 + 1],
        (1, 1) => map[point.1 + 1][point.0],
        _ => unreachable!()
    }.collect::<Vec<_>>()
}

fn calculate_isotropic_variants(
    map: &Arc<Vec<Vec<Vec<usize>>>>, 
    directions: &Vec<Direction>,
    relations: &Relations,
    point: Point
) -> Vec<usize>{
    let isotropic = relations.get_all_isotropic();
    if isotropic.is_empty(){
        return actual_variants;
    }

    // Get ruled neighbour variants for every possible variant of current 
    // point and intersect with current point variants

    let mut neighbours_variants = directions.iter().map(|d|{
        (d, get_neighbour_variants(map, point, d.difference()).into_iter().collect::<HashSet<_>>())
    }).collect::<HashMap<_, _>>();

    let mut all_variants = HashSet::<usize>::from(map[point.1][point.0].iter());
    let old_variants = all_variants;
    for d in directions{
        let mut neighbour_variants = neighbours_variants.get_mut(d).unwrap();
        for i in old_variants{
            if !all_variants.contains(&i){
                continue;
            }
            let Relation::Isotropic(limited_variants) = relations.0[i] else{
                unreachable!();
            };
            neighbour_variants = neighbour_variants.intersection(limited_variants).collect::<HashSet<_>>();
        }
    }
    for (_, variants) in neighbours_variants{
        all_variants = all_variants.intersection(&variants).collect::<HashSet<_>>();
    }

    all_variants.iter().collect::<Vec<_>>()
}

fn calculate_anisotropic_variants(
    map: &Arc<Vec<Vec<Vec<usize>>>>, 
    directions: &Vec<Direction>,
    relations: &Relations,
    point: Point
) -> Vec<usize>{
    let anisotropic = relations.get_all_anisotropic();
    if anisotropic.is_empty(){
        return actual_variants;
    }

    // Get ruled neighbour variants for every possible variant of current 
    // point and intersect with current point variants by every direction

    let mut neighbours_variants = directions.iter().map(|d|{
        (d, get_neighbour_variants(map, point, d.difference()).into_iter().collect::<HashSet<_>>())
    }).collect::<HashMap<_, _>>();

    let mut all_variants = HashSet::<usize>::from(map[point.1][point.0].iter());
    let old_variants = all_variants;
    for d in directions{
        let mut neighbour_variants = neighbours_variants.get_mut(d).unwrap();
        for i in old_variants{
            if !all_variants.contains(&i){
                continue;
            }
            let Relation::Anisotropic(limited_variants) = relations.0[i] else{
                unreachable!();
            };
            if let Some(variants) = limited_variants.get(d){
                neighbour_variants = neighbour_variants.intersection(variants)
                    .collect::<HashSet<_>>();
            }
        }
    }
    for (_, variants) in neighbours_variants{
        all_variants = all_variants.intersection(&variants).collect::<HashSet<_>>();
    }
    
    all_variants.iter().collect::<Vec<_>>()
}

fn get_neighbours_map(points: Vec<Point>) -> Vec<(Point, Vec<Point>)>{
    points.into_par_iter().map(|p|{
        let mut points = vec![];
        for (dim, diff) in diffs{
            points.push(match dim{
                0 => Point(p.0 + diff, p.1),
                1 => Point(p.0, p.1 + diff),
                _ => unreachable!()
            });
        }
        (p, points)
    }).collect::<Vec<_>>()
}

fn get_neighbours(points: Vec<Point>) -> Vec<Point>{
    points.into_par_iter().map(|(x,y): Point|{
        let mut points = vec![];
        for (dim, diff) in diffs{
            points.push(match dim{
                0 => Point(x + diff, y),
                1 => Point(x, y + diff),
                _ => unreachable!()
            });
        }
        points
    }).flatten().collect::<HashSet<_>>()
    .into_iter().collect::<Vec<_>>()
}

fn default_edges() -> HashMap<Direction, Vec<usize>>{
    let mut edges = HashMap::new();
    let directions = [
        Direction::N,
        Direction::S,
        Direction::W,
        Direction::E
    ];
    for d in directions{
        edges.insert(d, vec![]);
    }
    edges
}

fn choose_multiple<T>(range: Range<T>, count: usize) -> Vec<T>{
    (0..count)
        .into_par_iter()
        .map_init(|| SmallRng::from_rng(rand::thread_rng()), |r, _| {
            r.gen_range(range)
        }).collect::<Vec<T>>()
}

pub fn generate(
    size: (usize, usize), 
    relations: Relations, 
    density: f32
) -> Result<Vec<Vec<usize>>, &str>{
    if 0 > density && density < 1{
        return "density outside [0;1] range";
    }
    let random_count = ((size.0 * size.1) as f32 * density).floor();
    let x = choose_multiple::<usize>(0..size.0, random_count);
    let y = choose_multiple::<usize>(0..size.1, random_count);
    let v = choose_multiple::<usize>(0..relations.0.len(), random_count);
    let front = x.iter().zip(y.iter())
        .map(|(x, y)| Point(x, y)).collect()
        .iter().zip(v.iter()).collect();
    Wfc::new().size(size)
        .edges(default_edges())
        .front(front)
        .relations(relations)
        .calculate()
}