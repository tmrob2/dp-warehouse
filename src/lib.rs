use hashbrown::{HashSet, HashMap};
use pyo3::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json;

type State = ((i8, i8), u8, Option<(u8, u8)>);

#[derive(Serialize, Deserialize)]
struct Word {
    a: (i8, i8),
    c: u8,
    r: Option<(u8, u8)>
}

impl Word {
    fn new(a: (i8, i8), c: u8, r: Option<(u8, u8)>) -> Self {
        Self {a, c, r}
    }
}

fn pos_addition(
    pos: &(i8, i8), 
    dir: &(i8, i8),
    xmin: usize, 
    xmax: usize, 
    ymin: usize, 
    ymax: usize
) -> (i8, i8) {
    
    let x = if (pos.0 + dir.0) > xmin as i8 && (pos.0 + dir.0) < xmax as i8 - 1 {
        pos.0 + dir.0
    } else if (pos.0 + dir.0) < xmin as i8 {
        0 as i8
    } else {
        (xmax - 1) as i8
    };

    let y = if (pos.1 + dir.1) > ymin as i8 && (pos.1 + dir.1) < ymax as i8 - 1 {
        pos.1 + dir.1
    } else if (pos.1 + dir.1) < ymin as i8 {
        0 as i8
    } else {
        (ymax - 1) as i8
    };

    (x, y)
}

#[pyfunction]
fn place_racks(xsize: usize, ysize: usize) 
-> HashSet<(i8, i8)> {
    let mut racks: HashSet<(i8, i8)> = HashSet::new();
    let mut count: usize = 0;
    assert!(xsize > 5);
    assert!(ysize > 4);
    for c in 2..xsize - 2 {
        for r in 1..ysize - 2 {
            if count < 2 {
                count += 1;
                racks.insert((r as i8, c as i8));
            } else {
                count = 0;
            }
        }
    }
    racks
}

#[pyfunction]
fn set_state_space(
    init_state: ((i8, i8), u8, Option<(u8, u8)>), 
    nact: usize, 
    action_to_dir: HashMap<u8, (i8, i8)>,
    racks: HashSet<(i8, i8)>,
    grid_size: (usize, usize)
) -> (
    Vec<((i8, i8), u8, Option<(u8, u8)>)>, 
    HashMap<((i8, i8), u8, Option<(u8, u8)>), i32>,
    HashMap<i32, ((i8, i8), u8, Option<(u8, u8)>)>,
    HashMap<(i32, u8), Vec<(i32, f64, String)>>
) {
    let mut stack: Vec<State> = Vec::new();
    let mut state_map: HashMap<State, i32> = HashMap::new();
    let mut rev_state_map: HashMap<i32, State> = HashMap::new();
    // set a counter for the number of states involved in the system
    let mut state_count: i32 = 0; 
    let mut visited: HashSet<State> = HashSet::new();
    let mut transitions: HashMap<(i32, u8), Vec<(i32, f64, String)>> = HashMap::new();
    stack.push(init_state);
    state_map.insert(init_state, 0);
    rev_state_map.insert(0, init_state);
    state_count += 1;
    
    while !stack.is_empty() {
        let new_state = stack.pop().unwrap();
        let sidx = *state_map.get(&new_state).unwrap();

        for action in 0..nact {
            let v = step(
                new_state, 
                action as u8,
                &action_to_dir,
                grid_size.0,
                grid_size.1,
                &racks
            ).unwrap();
            //println!("s: {:?}, a: {} -> s': {:?}", new_state, action, v);
            for (sprime, _, _) in v.iter() {
                if !visited.contains(sprime) {
                    state_map.insert(*sprime, state_count);
                    rev_state_map.insert(state_count, *sprime);
                    state_count += 1;
                    visited.insert(*sprime);
                    stack.push(*sprime);
                }
            }
            let v_: Vec<(i32, f64, String)> = v.iter()
                .map(|(s, p, w)| (*state_map.get(s).unwrap(), *p, w.to_string()))
                .collect();
            transitions.insert((sidx as i32, action as u8), v_);
        }
    }
    
    let visited_: Vec<((i8, i8), u8, Option<(u8, u8)>)> = visited.into_iter().collect();
    (visited_, state_map, rev_state_map, transitions)
}

fn step(
    state: State, 
    action: u8, 
    action_to_dir: &HashMap<u8, (i8, i8)>, 
    xsize: usize, 
    ysize: usize,
    racks: &HashSet<(i8, i8)>
) -> Result<Vec<(State, f64, String)>, String> {

    let mut v: Vec<(State, f64, String)> = Vec::new();
    if vec![0, 1, 2, 3].contains(&action) {
        let direction: &(i8, i8) = action_to_dir.get(&action).unwrap();
        
        let agent_new_loc = pos_addition( &state.0, direction, 0, xsize, 0, ysize);
        
        if state.1 == 0 {
            let new_state = (agent_new_loc, 0, state.2);
            let word = Word::new(new_state.0, new_state.1, new_state.2);
            let serialised_word = serde_json::to_string(&word).unwrap();
            v.push((new_state, 1.0, serialised_word));
        } else {
            // an agent is restricted to move with corridors
            // check that the agent is in a corridor, if not, then
            // it cannot proceed in a direction that is not a corridor
            if racks.contains(&state.0) {
                if racks.contains(&agent_new_loc) {
                   let word = Word::new(state.0, state.1, state.2);
                   v.push((state, 1.0, serde_json::to_string(&word).unwrap()));
                } else {
                    // Define the failure scenario
                    let success_rack_pos = Some((agent_new_loc.0 as u8, agent_new_loc.1 as u8));
                    let success_state: State = (agent_new_loc, 1, success_rack_pos);
                    // The rack moves with the agent
                    let success_word = Word::new(agent_new_loc, 1, success_rack_pos); 
                    let fail_state: State = (agent_new_loc, 0, state.2);
                    let fail_word = Word::new(agent_new_loc, 0, state.2);
                    v.push((success_state, 0.99, serde_json::to_string(&success_word).unwrap()));
                    v.push((fail_state, 0.01, serde_json::to_string(&fail_word).unwrap()));
                }
            } else {
                // Define the failure scenario
                let success_rack_pos = Some((agent_new_loc.0 as u8, agent_new_loc.1 as u8));
                let success_state: State = (agent_new_loc, 1, success_rack_pos);
                // The rack moves with the agent
                let success_word = Word::new(agent_new_loc, 1, success_rack_pos); 
                let fail_state: State = (agent_new_loc, 0, state.2);
                let fail_word = Word::new(agent_new_loc, 0, state.2);
                v.push((success_state, 0.99, serde_json::to_string(&success_word).unwrap()));
                v.push((fail_state, 0.01, serde_json::to_string(&fail_word).unwrap()));
            }
        };
    } else if action == 4 {
        if state.1 == 0 {
            // if the agent is in a rack position then it may carry a rack
            // OR is the agent is not in a rack position but is superimposed on 
            // a rack that is in a corridor then it may pick up the rack. 
            let cmp_state: Option<(i8, i8)> = match state.2 {
                Some(val) => { Some((val.0 as i8, val.1 as i8)) }
                None => { None }
            };
            if racks.contains(&state.0) {
                let new_rack_pos = Some((state.0.0 as u8, state.0.1 as u8));
                let word = Word::new(state.0, 1, new_rack_pos);
                let new_state = (state.0, 1, new_rack_pos);
                v.push((new_state, 1.0, serde_json::to_string(&word).unwrap()));
            } else if cmp_state.is_some() {
                if cmp_state.unwrap() == state.0 {
                    let new_rack_pos = Some((state.0.0 as u8, state.0.1 as u8));
                    let word = Word::new(state.0, 1, new_rack_pos);
                    let new_state = (state.0, 1, new_rack_pos);
                    v.push((new_state, 1.0, serde_json::to_string(&word).unwrap()));
                } else {
                    let word = Word::new(state.0, 0, state.2);
                    let new_state = (state.0, 0, state.2);
                    v.push((new_state, 1.0, serde_json::to_string(&word).unwrap()));
                }
            } else {
                let word = Word::new(state.0, 0, state.2);
                let new_state = (state.0, 0, state.2);
                v.push((new_state, 1.0, serde_json::to_string(&word).unwrap()));
            }
        } else {
            let word = Word::new(state.0, 1, state.2);
            v.push((state, 1.0, serde_json::to_string(&word).unwrap()));
        }
    } else if action == 5 {
        if state.1 == 1 {
            // this agent is currently carrying something
            // therefore, drop the rack at the current agent position
            let new_rack_pos = Some((state.0.0 as u8, state.0.1 as u8));
            let word = Word::new(state.0, 0, new_rack_pos);
            v.push((state, 1.0, serde_json::to_string(&word).unwrap()));
        } else {
            let word = Word::new(state.0, 0, state.2);
            v.push((state, 1.0, serde_json::to_string(&word).unwrap()));
        }
    } else {
        return Err("action not registered".to_string())
    }

    Ok(v)
}

/// A Python module implemented in Rust.
#[pymodule]
fn dp_warehouse(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(set_state_space, m)?)?;
    m.add_function(wrap_pyfunction!(place_racks, m)?)?;
    Ok(())
}