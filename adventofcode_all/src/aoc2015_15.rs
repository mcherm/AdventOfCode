mod eznom;

use std::fs;
use std::io;
use std::cmp::max;
use itertools::Itertools;
use nom::multi::many0 as nom_many0;
use nom::character::complete::i32 as nom_value;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::newline as nom_newline;
use eznom::type_builder;


const RECIPE_SIZE: usize = 100;


#[derive(Debug, Clone)]
struct Ingredient {
    name: String,
    capacity: i32,
    durability: i32,
    flavor: i32,
    texture: i32,
    calories: i32,
}


const PROPERTIES: usize = 4; // capacity, durability, flavor, and texture


impl Ingredient {
    pub fn property_array(&self) -> [i32; PROPERTIES] {
        [self.capacity, self.durability, self.flavor, self.texture]
    }

    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_alpha1,
            nom_tag(": capacity "),
            nom_value,
            nom_tag(", durability "),
            nom_value,
            nom_tag(", flavor "),
            nom_value,
            nom_tag(", texture "),
            nom_value,
            nom_tag(", calories "),
            nom_value,
            nom_newline,
        ))(s);
        let build =
            |
                (name, _, capacity, _, durability, _, flavor, _, texture, _, calories, _):
                (&str, &str, i32, &str, i32, &str, i32, &str, i32, &str, i32, char)
            |
                Ingredient{
                    name: name.to_string(),
                    capacity,
                    durability,
                    flavor,
                    texture,
                    calories,
                }
            ;
        type_builder(recognize, build)(input)
    }
}



fn input() -> Result<Vec<Ingredient>, io::Error> {
    let s = fs::read_to_string("input/2015/15/input.txt")?;
    match Ingredient::parse_list(&s) {
        Ok(("", ingredients)) => Ok(ingredients),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


/// Given a list of ingredients and amounts, this returns the score for that cookie
fn score(recipe: &Vec<(&Ingredient, i32)>) -> i32 {
    assert!(recipe.iter().all(|(_,a)| *a>= 0 && *a <= 100));
    assert!(recipe.iter().map(|(_,a)| a).sum::<i32>() == 100);

    let mut property_sums: [i32; PROPERTIES] = [0; PROPERTIES];
    for (ingredient, amount) in recipe.iter() {
        let properties = ingredient.property_array();
        for i in 0..PROPERTIES {
            property_sums[i] += properties[i] * amount;
        }
    }
    property_sums.iter().map(|x| max(*x, 0)).product()
}


/// This creates an iterator that yields valid amounts for recipes. It
/// yields arrays of length len of u32 where each value is >= 0 and all
/// the values sum to sum.
#[derive(Debug)]
struct IngredientPermutationIter {
    len: usize,
    sum: i32,
    is_exhausted: bool,
    next_value: Vec<i32>,
}

impl IngredientPermutationIter {
    fn new(len: usize, sum: i32) -> Self {
        let is_exhausted = false;
        let mut next_value = vec![0; len];
        next_value[len - 1] = sum;
        IngredientPermutationIter{len, sum, is_exhausted, next_value}
    }

    fn increment(&mut self) {
        if !self.is_exhausted {
            let last_col = self.len - 1;
            let mut col = last_col - 1;
            loop {
                let sum_of_prev: i32 = self.next_value[..col].iter().sum();
                if self.next_value[col] < self.sum - sum_of_prev {
                    self.next_value[col] += 1;
                    self.next_value[last_col] -= 1;
                    assert!( self.next_value[last_col] >= 0 );
                    break;
                } else {
                    if col > 0 {
                        self.next_value[col] = 0;
                        self.next_value[last_col] = self.sum - sum_of_prev;
                        col -= 1;
                    } else {
                        self.is_exhausted = true;
                        break;
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for IngredientPermutationIter {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let answer = if self.is_exhausted {
            None
        } else {
            Some(self.next_value.clone())
        };
        self.increment();
        answer
    }
}



fn part_a(ingredients: &Vec<Ingredient>) -> Result<(), io::Error> {
    assert!(ingredients.len() > 0);
    let mut high_score = -1; // invalid, but it will be replaced
    let mut high_recipe: Vec<(&Ingredient, i32)> = Vec::new(); // invalid, but it will be replaced
    let ipi = IngredientPermutationIter::new(ingredients.len(), RECIPE_SIZE as i32);
    for amounts in ipi {
        let items_and_amounts: Vec<(&Ingredient, i32)> = ingredients.iter().zip_eq(amounts).collect();
        let s = score(&items_and_amounts);
        if s > high_score {
            high_score = s;
            high_recipe = items_and_amounts;
        }
    }
    let recipe_description = high_recipe.iter().map(|(i,a)| format!("{}tsp of {}", a, i.name)).join(", ");
    println!("The highest score is: {} which comes from {}", high_score, recipe_description);
    Ok(())
}


fn part_b(ingredients: &Vec<Ingredient>) -> Result<(), io::Error> {
    assert!(ingredients.len() > 0);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
