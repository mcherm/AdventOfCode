
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fs;
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        combinator::map,
        character::complete::line_ending,
        sequence::{tuple, terminated},
        multi::many0,
    };
    use nom::character::complete::u32 as nom_Num;
    use std::collections::{VecDeque, HashSet};
    use std::fmt::{Display, Formatter};


    const PRINT: bool = false;
    const CHECK_INVARIANTS: bool = false;


    pub fn input() -> Result<Object3D, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_18.txt")?;
        match Object3D::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Point3D {
        x: u32,
        y: u32,
        z: u32,
    }


    #[derive(Debug)]
    pub struct Object3D {
        pt_set: HashSet<Point3D>
    }

    struct NeighborsIter {
        i: u8,
        p: Point3D,
    }


    impl Point3D {
        /// Construtor. Add 1 in all directions so we won't need to worry about a neighbor being
        /// out of bounds.
        fn new_offset(x: u32, y: u32, z: u32) -> Self {
            Point3D{x: x+1, y: y+1, z: z+1}
        }

        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            map(
                tuple((
                    nom_Num,
                    tag(","),
                    nom_Num,
                    tag(","),
                    nom_Num
                )),
                |(x,_,y,_,z)| Point3D::new_offset(x,y,z)
            )(input)
        }

        /// Returns an iterator of the points neighboring this one. 6, unless it's
        /// beside 0.
        ///
        /// This guarantees that the neighbors with LOWER values of x,y,z will occur
        /// before the neighbors with a HIGHER value of these (and the flood algorithm
        /// depends on that guarantee).
        fn neighbors(&self) -> impl Iterator<Item=Self> {
            NeighborsIter{i: 0, p: *self}
        }

        /// Returns true if any dimension of this point has the value 0.
        fn has_zero(&self) -> bool {
            self.x == 0 || self.y == 0 || self.z == 0
        }
    }

    impl Display for Point3D {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({:2},{:2},{:2})", self.x, self.y, self.z)
        }
    }

    impl Object3D {
        /// Create an Object
        fn new(pts: Vec<Point3D>) -> Self {
            let pt_set: HashSet<Point3D> = pts.iter().copied().collect();
            assert!(pt_set.len() == pts.len()); // if not, there was a dup
            Object3D{pt_set}
        }


        /// Parses a newline-terminated list of LineSpecs
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            map(
                many0(terminated(Point3D::parse, line_ending)),
                |pts| Object3D::new(pts)
            )(input)
        }

        /// Finds the surface area
        pub fn surface_area(&self) -> u32 {
            self.pt_set.iter()
                .map(|p| p.neighbors().filter(|n| !self.pt_set.contains(n)).count() as u32)
                .sum()
        }

        /// Finds the outer surface area
        ///
        /// Strategy: find all neighbor points. Classify them as "outer" or "inner" by
        /// taking one and attempting to flood fill it with a bias to lower coordinates and see
        /// if we can hit 0 in ANY coordinate (which is definitely "outer"). Other things found
        /// during the flood are also classified at the same time (including some that aren't
        /// neighbors). Once we're done classifying all neighbors, THEN we can count only surfaces
        /// that border an outer neighbor.
        pub fn outer_surface_area(&self) -> u32 {
            let body_pts = &self.pt_set;
            let mut nbr_pts: HashSet<Point3D> = body_pts.iter().flat_map(|p| p.neighbors()).collect();
            let mut outer_pts: HashSet<Point3D> = HashSet::new();
            let mut inner_pts: HashSet<Point3D> = HashSet::new();

            // Loop until all neighbors have been classified
            while !nbr_pts.is_empty() {
                // Pick a point in nbr_pts and move it to flood_edge_pts.
                let start_pt: Point3D = nbr_pts.iter().next().unwrap().clone();
                nbr_pts.remove(&start_pt);

                // If we haven't already classified this point
                if body_pts.contains(&start_pt) || outer_pts.contains(&start_pt) || inner_pts.contains(&start_pt) {
                    // This one is already classified
                    continue;
                } else {
                    // Flood outward from there, and add to outer_pts or to inner_pts
                    flood_from_point(start_pt, body_pts, &mut outer_pts, &mut inner_pts);
                }
            }

            // Now we know which are outer neighbors and we can count sides based on that.
            body_pts.iter()
                .map(|p| p.neighbors().filter(|n| outer_pts.contains(n)).count() as u32)
                .sum()
        }
    }

    #[derive(Debug, Copy, Clone)]
    enum Containment { Outer, Inner }

    /// An internal subroutine of outer_surface_area, this starts from the given start_pt
    /// and floods outward (preferring lower numbers first) avoiding body_pts until it either
    /// hits a zero in some coordinate (it's outer) or runs out of edge for the flood (it's
    /// inner). Then all the points it encountered are added to outer_pts or to inner_pts.
    fn flood_from_point(start_pt: Point3D, body_pts: &HashSet<Point3D>, outer_pts: &mut HashSet<Point3D>, inner_pts: &mut HashSet<Point3D>) {
        // Handle special case of already being on the edge - no flooding needed
        if start_pt.has_zero() {
            outer_pts.insert(start_pt);
            return;
        }
        assert!( !start_pt.has_zero() );

        // To do the flood we'll track "edge" and "core" points.
        let mut flood_edge_queue: VecDeque<Point3D> = VecDeque::new();
        let mut flood_edge_pts: HashSet<Point3D> = HashSet::new();
        let mut flood_core_pts: HashSet<Point3D> = HashSet::new();

        // Start with the one point we were told to start on
        flood_edge_queue.push_back(start_pt);
        flood_edge_pts.insert(start_pt);

        // Flood until we find out whether it's inner or outer
        let containment = flood_from_point_inner(body_pts, outer_pts, inner_pts, &mut flood_edge_queue, &mut flood_edge_pts, &mut flood_core_pts);

        if PRINT {
            println!("Found a group of {} that are {:?}.", flood_core_pts.len() + flood_edge_pts.len(), containment);
        }

        // Having found where they go, mark the flood core pts as Inner/Outer
        match containment {
            Containment::Outer => {
                outer_pts.extend(flood_core_pts.iter());
                outer_pts.extend(flood_edge_pts.iter());
            },
            Containment::Inner => {
                inner_pts.extend(flood_core_pts.iter());
                inner_pts.extend(flood_edge_pts.iter());
            },
        }
    }

    fn flood_from_point_inner(
        body_pts: &HashSet<Point3D>,
        outer_pts: &HashSet<Point3D>,
        inner_pts: &HashSet<Point3D>,
        flood_edge_queue: &mut VecDeque<Point3D>,
        flood_edge_pts: &mut HashSet<Point3D>,
        flood_core_pts: &mut HashSet<Point3D>
    ) -> Containment {
        // Loop until nothing is left in the edge set (it's "inner") or we reach 0 in some
        // coordinate (it's "outer").
        while !flood_edge_pts.is_empty() {

            if CHECK_INVARIANTS { // some asserts to check on invariants
                // Assert edge_queue == edge_pts
                let edge_copy: HashSet<Point3D> = flood_edge_queue.iter().cloned().collect();
                assert_eq!(flood_edge_pts.clone(), edge_copy);
                // Assert edge and core are unique and distinct
                let mut flood_list: Vec<Point3D> = flood_core_pts.iter().cloned().collect();
                flood_list.extend(flood_edge_queue.iter());
                let flood_set: HashSet<Point3D> = flood_list.iter().cloned().collect();
                if flood_list.len() != flood_set.len() {
                    println!("{:?}", flood_list);
                }
                assert_eq!(flood_list.len(), flood_set.len());
                // Assert none of edge or core has zeros
                assert!( flood_edge_pts.iter().all(|x| !x.has_zero()) );
                assert!( flood_core_pts.iter().all(|x| !x.has_zero()) );
            }

            let next_flood_pt: Point3D = flood_edge_queue.pop_front().unwrap();
            flood_edge_pts.remove(&next_flood_pt);
            flood_core_pts.insert(next_flood_pt);
            if PRINT {
                if flood_core_pts.len() % 100 == 0 {
                    println!("    core size reached {}", flood_core_pts.len());
                }
            }
            assert!(!next_flood_pt.has_zero());
            for neighbor in next_flood_pt.neighbors() {
                if neighbor.has_zero() {
                    return Containment::Outer; // we reached zero. This is outer!
                } else if body_pts.contains(&neighbor) {
                    // found a part of the body. Ignore it
                } else if outer_pts.contains(&neighbor) {
                    return Containment::Outer; // connected with the outside. This is outer!
                } else if inner_pts.contains(&neighbor) {
                    return Containment::Inner; // connected with the inside. This is inner!
                } else if flood_edge_pts.contains(&neighbor) || flood_core_pts.contains(&neighbor) {
                    // It's already part of the flood. Ignore it
                } else {
                    // otherwise, add it to the edge queue
                    flood_edge_pts.insert(neighbor);
                    flood_edge_queue.push_back(neighbor);
                }
            }
        }
        return Containment::Inner;
    }


    impl Iterator for NeighborsIter {
        type Item = Point3D;

        fn next(&mut self) -> Option<Self::Item> {
            self.i += 1;
            match self.i {
                0 => panic!(),
                1 => Some(Point3D{x: self.p.x - 1, ..self.p}),
                2 => Some(Point3D{y: self.p.y - 1, ..self.p}),
                3 => Some(Point3D{z: self.p.z - 1, ..self.p}),
                4 => Some(Point3D{x: self.p.x + 1, ..self.p}),
                5 => Some(Point3D{y: self.p.y + 1, ..self.p}),
                6 => Some(Point3D{z: self.p.z + 1, ..self.p}),
                _ => None,
            }
        }
    }
}



// ======= Part 1 Compute =======

mod compute {

}




// ======= main() =======

use crate::parse::{Object3D, input};


fn part_a(input: &Object3D) {
    println!("\nPart a:");
    println!("The surface area is {}", input.surface_area())
}


fn part_b(input: &Object3D) {
    println!("\nPart b:");
    println!("The outer surface area is {}", input.outer_surface_area())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

