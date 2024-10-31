use std::{
    collections::{HashMap, HashSet},
    ops::Not,
};

use axum::{routing::post, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/integers", post(task1))
        .route("/rocket", post(task2))
}

pub async fn task1(body: String) -> String {
    let nums =
        body.lines()
            .map(|num| num.parse::<u64>().unwrap())
            .fold(HashSet::new(), |mut map, num| {
                map.remove(&num).not().then(|| map.insert(num));
                map
            });
    let val = nums.iter().next().unwrap();
    "🎁".repeat(*val as usize)
}

pub async fn task2(body: String) -> String {
    let mut lines = body.lines();
    let num_stars = lines.next().unwrap().parse::<i32>().unwrap();
    let stars: Vec<_> = lines
        .clone()
        .take(num_stars as usize)
        .map(|line| {
            line.split(" ")
                .map(|x| x.parse::<i32>().unwrap())
                .collect::<Vec<_>>()
        })
        .collect();
    let mut lines = lines.skip(num_stars as usize).clone();
    let num_portals: i32 = lines.next().unwrap().parse().unwrap();
    let portals = lines
        .take(num_portals as usize)
        .map(|line| {
            line.split(" ")
                .map(|x| x.parse::<i32>().unwrap())
                .collect::<Vec<_>>()
        })
        .fold(HashMap::new(), |mut acc, portal| {
            acc.entry(portal[0])
                .and_modify(|x: &mut Vec<_>| x.push(portal[1]))
                .or_default()
                .push(portal[1]);
            acc
        });
    let path = bfs(&portals, 0, num_stars - 1);
    let output = format!(
        "{} {:.3}",
        path.len() - 1,
        calculate_distance(&path, &stars)
    );
    output
}

fn bfs(graph: &HashMap<i32, Vec<i32>>, start: i32, end: i32) -> Vec<i32> {
    let mut queue = Vec::new();
    queue.push(start);
    let mut visited = HashSet::new();
    let mut edge_to = HashMap::new();
    visited.insert(start);
    while !queue.is_empty() {
        let node = queue.remove(0);
        if node == end {
            break;
        }
        for &next in graph.get(&node).unwrap_or(&Vec::new()) {
            if !visited.contains(&next) {
                visited.insert(next);
                queue.push(next);
                edge_to.insert(next, node);
            }
        }
    }
    let mut path = vec![start];
    let mut x = end;
    while x != start {
        path.insert(1, x);
        x = edge_to[&x];
    }
    path
}

fn calculate_distance(path: &[i32], stars: &[Vec<i32>]) -> f32 {
    path.iter()
        .zip(path.iter().skip(1))
        .map(|(a, b)| distance(&stars[*a as usize], &stars[*b as usize]))
        .sum()
}

fn distance(pos1: &[i32], pos2: &[i32]) -> f32 {
    f32::sqrt(
        (pos1[0].abs_diff(pos2[0]).pow(2)
            + pos1[1].abs_diff(pos2[1]).pow(2)
            + pos1[2].abs_diff(pos2[2]).pow(2)) as f32,
    )
}
