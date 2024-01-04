use std::collections::HashMap;
use std::fs;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

static NUM_TRIALS: f64 = 10000000f64;

// TODO Take into account that you can open multiple different chests (dungeon chest keys)

#[derive(Deserialize)]
struct Loot {
    #[serde(rename = "displayName")]
    display_name: String,
    id: String,
    chance: f64,
    #[serde(rename = "maxScore")]
    max_score: f64
}
type Floor = Vec<Loot>;
type DataDump = HashMap<String, Floor>;

struct GoodFloor {
    floor: String,
    loot: Vec<Loot>
}

#[derive(Serialize)]
struct SimulatedLoot {
    display_name: String,
    id: String,
    max_score: f64,
    base_chance: f64,
    meter_s_chance: f64,
    meter_s_plus_chance: f64,
    base_reroll_chance: f64,
    base_reroll_amount_per_drop: f64,
    meter_s_reroll_chance: f64,
    meter_s_reroll_amount_per_drop: f64,
    meter_s_plus_reroll_chance: f64,
    meter_s_plus_reroll_amount_per_drop: f64
}

fn get_drop_rate(base_drop_rate: &f64, stored_score: &f64, max_score: &f64) -> f64 {
    if stored_score >= max_score {
        1f64
    }else {
        // https://hypixel-skyblock.fandom.com/wiki/RNG_Meter
        let extra_multiplier = 2f64 * (stored_score / max_score) / 100f64;
        let multiplier = 1f64 + extra_multiplier;
        base_drop_rate * multiplier
    }
}

fn main() {
    let data_dump: DataDump = serde_json::from_str(&fs::read_to_string("./dump.json").unwrap()).unwrap();
    let mut good_floors = Vec::with_capacity(data_dump.len());
    for entry in data_dump {
        good_floors.push(GoodFloor {
            floor: entry.0,
            loot: entry.1
        });
    }

    let mut out: Vec<(String, Vec<SimulatedLoot>)> = good_floors.par_iter().map(|el| {
        (el.floor.clone(), el.loot.par_iter().map(|loot| {
            let base_chance = loot.chance;
            let max_score = loot.max_score;
            let meter_s_chance = {
                let mut count = 0f64;
                let mut score = 0f64;
                for _ in 0..(NUM_TRIALS as usize) {
                    score += 270f64 * 0.7f64; // S Runs only grant 70% score
                    if fastrand::f64() < get_drop_rate(&base_chance, &score, &max_score) {
                        count += 1f64;
                        score = 0f64;
                    }
                }

                count / NUM_TRIALS
            };
            let meter_s_plus_chance = {
                let mut count = 0f64;
                let mut score = 0f64;
                for _ in 0..(NUM_TRIALS as usize) {
                    score += 300f64;
                    if fastrand::f64() < get_drop_rate(&base_chance, &score, &max_score) {
                        count += 1f64;
                        score = 0f64;
                    }
                }

                count / NUM_TRIALS
            };
            let (base_reroll_chance, base_reroll_amount_per_drop) = {
                let mut count = 0f64;
                let mut rerolls = 0f64;
                for _ in 0..(NUM_TRIALS as usize) {
                    if fastrand::f64() < base_chance {
                        count += 1f64;
                    } else {
                        rerolls += 1f64;
                        if fastrand::f64() < base_chance {
                            count += 1f64;
                        }
                    }
                }

                (count / NUM_TRIALS, rerolls / count)
            };
            let (meter_s_reroll_chance, meter_s_reroll_amount_per_drop) = {
                let mut count = 0f64;
                let mut score = 0f64;
                let mut rerolls = 0f64;
                for _ in 0..(NUM_TRIALS as usize) {
                    score += 270f64 * 0.7f64; // S Runs only grant 70% score
                    if fastrand::f64() < get_drop_rate(&base_chance, &score, &max_score) {
                        count += 1f64;
                        score = 0f64;
                    } else {
                        rerolls += 1f64;
                        if fastrand::f64() < base_chance { // Meter doesn't apply on reroll
                            count += 1f64;
                            score = 0f64;
                        }
                    }
                }

                (count / NUM_TRIALS, rerolls / count)
            };
            let (meter_s_plus_reroll_chance, meter_s_plus_reroll_amount_per_drop) = {
                let mut count = 0f64;
                let mut score = 0f64;
                let mut rerolls = 0f64;
                for _ in 0..(NUM_TRIALS as usize) {
                    score += 300f64;
                    if fastrand::f64() < get_drop_rate(&base_chance, &score, &max_score) {
                        count += 1f64;
                        score = 0f64;
                    } else {
                        rerolls += 1f64;
                        if fastrand::f64() < base_chance { // Meter doesn't apply on reroll
                            count += 1f64;
                            score = 0f64;
                        }
                    }
                }

                (count / NUM_TRIALS, rerolls / count)
            };

            SimulatedLoot {
                display_name: loot.display_name.clone(),
                id: loot.id.clone(),
                max_score,
                base_chance,
                meter_s_chance,
                meter_s_plus_chance,
                base_reroll_chance,
                base_reroll_amount_per_drop,
                meter_s_reroll_chance,
                meter_s_reroll_amount_per_drop,
                meter_s_plus_reroll_chance,
                meter_s_plus_reroll_amount_per_drop
            }
        }).collect())
    }).collect();

    let mut hashmap_out: HashMap<String, Vec<SimulatedLoot>> = HashMap::with_capacity(out.len());
    for _ in 0..out.len() {
        let el = out.remove(0);
        hashmap_out.insert(el.0, el.1);
    }

    fs::write("./out.json", serde_json::to_string(&hashmap_out).unwrap()).unwrap();
    fs::write("./out_pretty.json", serde_json::to_string_pretty(&hashmap_out).unwrap()).unwrap();
}
