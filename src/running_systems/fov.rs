// All algorithms and much of the notation has been lifted from the extraordinary blog post:
// http://journal.stuffwithstuff.com/2015/09/07/what-the-hero-sees/
// Except there were weird bugs.
//
// Instead of figuring out how octants + overlap are supposed to work, I used the same shadow line
// concept, but with radial shadows instead -- the shadows are rays, not slopes, so we don't need
// octants and we can go around the circle all at once.

use std::cmp::Ordering;
use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::NotNan;

use crate::components::*;
use crate::map::*;

const FOV_DEBUGGING: bool = false;

#[derive(Copy, Clone, Hash, Debug)]
struct Ray {
    x: NotNan<f32>,
    y: NotNan<f32>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
// don't swap the order; right comes before left
enum RaySide {
    // includes x > 0 and vertical up
    Right,
    // includes x < 0 and vertical down
    Left,
}

#[inline(always)]
fn nn(f: f32) -> NotNan<f32> {
    NotNan::new(f).unwrap()
}

impl Ray {
    fn new(x: f32, y: f32) -> Ray {
        if x == 0.0 && y >= 0.0 {
            panic!("Cannot make a ray from zero, and cannot make an 'up' ray");
        }

        Ray { x: nn(x), y: nn(y) }
    }

    fn tangent(&self) -> NotNan<f32> {
        if self.x == nn(0.0) {
            if self.y > nn(0.0) {
                NotNan::new(f32::INFINITY).unwrap()
            } else {
                NotNan::new(f32::NEG_INFINITY).unwrap()
            }
        } else {
            self.y / self.x
        }
    }

    fn side(&self) -> RaySide {
        if self.x == nn(0.0) {
            if self.y > nn(0.0) {
                RaySide::Right
            } else {
                RaySide::Left
            }
        } else if self.x > nn(0.0) {
            RaySide::Right
        } else {
            RaySide::Left
        }
    }
}

impl PartialEq for Ray {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Ray {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Ray {}

impl Ord for Ray {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_side = self.side();
        let their_side = other.side();

        if my_side != their_side {
            return my_side.cmp(&their_side);
        }

        let my_tan = self.tangent();
        let their_tan = other.tangent();

        their_tan.cmp(&my_tan)
    }
}

#[derive(Copy, Clone, Debug)]
enum RadialShadow {
    FullCircle,
    // includes up on the left (start), but not the right (end)
    LeftClosed { end: Ray },
    // includes up on the right (end), but not the left (start)
    RightClosed { start: Ray },
    // doesn't include up on either end
    Open { start: Ray, end: Ray },
}

impl RadialShadow {
    fn starts_strictly_before(&self, other: &RadialShadow) -> bool {
        match self.left() {
            None => other.left().is_some(),
            Some(my_left) => match other.left() {
                None => false,
                Some(their_left) => my_left < their_left,
            },
        }
    }

    fn left(&self) -> Option<Ray> {
        match *self {
            RadialShadow::FullCircle => None,
            RadialShadow::LeftClosed { .. } => None,
            RadialShadow::RightClosed { start } => Some(start),
            RadialShadow::Open { start, end: _ } => Some(start),
        }
    }

    fn contains(&self, other: &RadialShadow) -> bool {
        match self {
            RadialShadow::FullCircle => true,
            RadialShadow::LeftClosed { end: my_end } => match other {
                RadialShadow::FullCircle => false,
                RadialShadow::RightClosed { .. } => false,
                RadialShadow::LeftClosed { end: their_end } => my_end >= their_end,
                RadialShadow::Open {
                    start: _,
                    end: their_end,
                } => my_end >= their_end,
            },
            RadialShadow::RightClosed { start: my_start } => match other {
                RadialShadow::FullCircle => false,
                RadialShadow::LeftClosed { .. } => false,
                RadialShadow::RightClosed { start: their_start } => my_start <= their_start,
                RadialShadow::Open {
                    start: their_start,
                    end: _,
                } => my_start <= their_start,
            },
            RadialShadow::Open {
                start: my_start,
                end: my_end,
            } => match other {
                RadialShadow::FullCircle => false,
                RadialShadow::LeftClosed { .. } => false,
                RadialShadow::RightClosed { .. } => false,
                RadialShadow::Open {
                    end: their_end,
                    start: their_start,
                } => my_start <= their_start && my_end >= their_end,
            },
        }
    }

    fn overlaps(&self, other: &RadialShadow) -> bool {
        match self {
            RadialShadow::FullCircle => true,
            RadialShadow::LeftClosed { end: my_end } => match other {
                RadialShadow::FullCircle => true,
                RadialShadow::RightClosed { start: their_start } => their_start <= my_end,
                RadialShadow::LeftClosed { end: _ } => true,
                RadialShadow::Open {
                    start: their_start,
                    end: _,
                } => their_start <= my_end,
            },
            RadialShadow::RightClosed { start: my_start } => match other {
                RadialShadow::FullCircle => true,
                RadialShadow::LeftClosed { end: their_end } => their_end >= my_start,
                RadialShadow::RightClosed { start: _ } => true,
                RadialShadow::Open {
                    start: _,
                    end: their_end,
                } => their_end >= my_start,
            },
            RadialShadow::Open {
                start: my_start,
                end: my_end,
            } => match other {
                RadialShadow::FullCircle => true,
                RadialShadow::LeftClosed { end: their_end } => my_start <= their_end,
                RadialShadow::RightClosed { start: their_start } => their_start <= my_end,
                RadialShadow::Open {
                    end: their_end,
                    start: their_start,
                } => {
                    let inner_min = Ord::max(my_start, their_start);
                    let inner_max = Ord::min(my_end, their_end);
                    inner_min <= inner_max
                }
            },
        }
    }

    /// Grow this shadow to include the other.
    /// Note that they must overlap or this may be impossible (function will not panic, just produce
    /// wrong output).
    fn include(&mut self, other: RadialShadow) {
        *self = match *self {
            RadialShadow::FullCircle => RadialShadow::FullCircle,
            RadialShadow::LeftClosed { end: my_end } => match other {
                RadialShadow::FullCircle => RadialShadow::FullCircle,
                RadialShadow::RightClosed { .. } => RadialShadow::FullCircle,
                RadialShadow::LeftClosed { end: their_end } => RadialShadow::LeftClosed {
                    end: Ord::max(my_end, their_end),
                },
                RadialShadow::Open {
                    start: _,
                    end: their_end,
                } => RadialShadow::LeftClosed {
                    end: Ord::max(my_end, their_end),
                },
            },
            RadialShadow::RightClosed { start: my_start } => match other {
                RadialShadow::FullCircle => RadialShadow::FullCircle,
                RadialShadow::LeftClosed { .. } => RadialShadow::FullCircle,
                RadialShadow::RightClosed { start: their_start } => RadialShadow::RightClosed {
                    start: Ord::min(my_start, their_start),
                },
                RadialShadow::Open {
                    start: their_start,
                    end: _,
                } => RadialShadow::RightClosed {
                    start: Ord::min(my_start, their_start),
                },
            },
            RadialShadow::Open {
                start: my_start,
                end: my_end,
            } => match other {
                RadialShadow::FullCircle => RadialShadow::FullCircle,
                RadialShadow::LeftClosed { end: their_end } => RadialShadow::LeftClosed {
                    end: Ord::max(my_end, their_end),
                },
                RadialShadow::RightClosed { start: their_start } => RadialShadow::RightClosed {
                    start: Ord::min(my_start, their_start),
                },
                RadialShadow::Open {
                    start: their_start,
                    end: their_end,
                } => RadialShadow::Open {
                    start: Ord::min(my_start, their_start),
                    end: Ord::max(my_end, their_end),
                },
            },
        };
    }
}

struct RadialShadowLine {
    shadows: Vec<RadialShadow>,
}

impl RadialShadowLine {
    /// Construct a new shadow line
    fn new() -> Self {
        RadialShadowLine {
            // Sorted list of shadows.
            // Contract: if i is a valid index, then shadows[i].start <= shadows[i].end
            // Contract: if i and i+1 are valid indexes, then shadows[i].end < shadows[i+1].start
            shadows: Vec::new(),
        }
    }

    /// Determines whether an existing shadow is completely covered by things in this shadow line.
    fn is_in_shadow(&self, projection: &RadialShadow) -> bool {
        for shadow in &self.shadows {
            if shadow.contains(projection) {
                return true;
            }
        }

        false
    }

    /// Adjust the shadow line to include the given shadow.
    /// This will insert the shadow carefully into the internal sorted list, absorbing
    /// and collapsing any adjacent shadows.
    ///
    /// PRE: self.is_in_shadow(shadow) is FALSE
    fn add(&mut self, mut shadow: RadialShadow) {
        if FOV_DEBUGGING {
            println!("Start state: {:?}", self.shadows);
            println!("Adding shadow {:?}", shadow);
        }
        // First, figure out where to slot the new shadow
        // This is the first point where shadow.start < self.shadows[index].start, or if
        // there is no such place, it's just self.shadows.len(), meaning stick it at the end
        let mut index = 0;
        while index < self.shadows.len() {
            if RadialShadow::starts_strictly_before(&self.shadows[index], &shadow) {
                break;
            }
            index += 1;
        }

        // As long as we overlap with the shadow to the left, absorb that shadow into this one
        // and pop that shadow out of the vector (adjusting the index accordingly).
        //
        // Note that this loop only ever runs once, but I'd rather leave it in "just in case"
        // than prove it and maintain it
        while index > 0 && RadialShadow::overlaps(&self.shadows[index - 1], &shadow) {
            shadow.include(self.shadows[index - 1]);
            index -= 1;
            self.shadows.remove(index);
        }

        // As long as we overlap with the shadow to the right, absorb that shadow into this one
        // and pop that shadow out of the vector. Note no index adjustment is required.
        //
        // In theory we can overlap with an arbitrary number of shadows to the right, although
        // due to how this function is currently called, this never actually happens
        while index < self.shadows.len() && RadialShadow::overlaps(&self.shadows[index], &shadow) {
            shadow.include(self.shadows[index]);
            self.shadows.remove(index);
        }

        // Now we can just stick it in there.
        self.shadows.insert(index, shadow);
        if FOV_DEBUGGING {
            println!("End state {:?}", self.shadows);
        }
    }

    // Determine if this shadow line is complete (this is a useful early stopping condition).
    fn is_full_shadow(&self) -> bool {
        self.shadows.len() == 1 && matches!(self.shadows[0], RadialShadow::FullCircle)
    }
}

fn project_tile_radially(relative_tile_pos: WorldPos) -> (RadialShadow, Option<RadialShadow>) {
    // a tile at (x, y) is assumed to cover the four points in (x - 0.5, y - 0.5) .. (x+0.5, y+0.5)
    // this is because the viewer is assumed to be in the center of their square
    // so if the tile in question is directly above the viewer, the arc goes across "straight up"
    // and we have to split it into two shadows

    // special case -- same tile, so it completely obstructs
    if relative_tile_pos.x == 0 && relative_tile_pos.y == 0 {
        return (RadialShadow::FullCircle, None);
    }

    let x = relative_tile_pos.x as f32;
    let y = relative_tile_pos.y as f32;

    // determine whether the tile in question crosses the "up" axis, which determines if
    // we need one shadow or two to represent it
    let crosses_up = x == 0.0 && y > 0.0;
    if crosses_up {
        // if it does cross up, then we know exactly what the relevant rays are (lower left and lower right)
        let ll = Ray::new(x - 0.5, y - 0.5);
        let left_shadow = RadialShadow::RightClosed { start: ll };

        let lr = Ray::new(x + 0.5, y - 0.5);
        let right_shadow = RadialShadow::LeftClosed { end: lr };

        (left_shadow, Some(right_shadow))
    } else {
        // if it doesn't cross up, then the arc is normally representable, and the sort "just works"
        let mut rays = Vec::new();
        rays.push(Ray::new(x - 0.5, y - 0.5));
        rays.push(Ray::new(x + 0.5, y - 0.5));
        rays.push(Ray::new(x - 0.5, y + 0.5));
        rays.push(Ray::new(x + 0.5, y + 0.5));
        rays.sort();

        let start = rays[0];
        let end = rays[3];

        if FOV_DEBUGGING {
            println!(
                "From wp {:?}, sorted ray array was {:?}",
                relative_tile_pos, rays
            );
        }

        (RadialShadow::Open { start, end }, None)
    }
}

fn refresh_area(hero: WorldPos, range: f32, map: &Map) -> HashSet<WorldPos> {
    fn sq_dist(wp: WorldPos) -> i32 {
        wp.x * wp.x + wp.y * wp.y
    }

    let range_ceil = range.ceil() as i32;
    let mut to_process = Vec::new();
    for x in -range_ceil..range_ceil + 1 {
        for y in -range_ceil..range_ceil + 1 {
            let offset_wp = WorldPos { x, y };
            to_process.push(offset_wp);
        }
    }
    to_process = to_process
        .iter()
        .copied()
        .filter(|wp| sq_dist(*wp) as f32 <= range * range)
        .collect();
    to_process.sort_by(|a, b| sq_dist(*a).cmp(&sq_dist(*b)));

    let mut shadow_line = RadialShadowLine::new();
    let mut visible = HashSet::new();
    for offset_wp in to_process {
        let (shadow_a, maybe_shadow_b) = project_tile_radially(offset_wp);

        if FOV_DEBUGGING {
            println!(
                "From coordinate {:?}, got shadow {:?}",
                offset_wp,
                (shadow_a, maybe_shadow_b)
            );
        }

        let is_blocked = shadow_line.is_in_shadow(&shadow_a)
            && maybe_shadow_b
                .map(|shadow_b| shadow_line.is_in_shadow(&shadow_b))
                .unwrap_or(true);
        if is_blocked {
            continue;
        }

        let actual_wp = WorldPos {
            x: offset_wp.x + hero.x,
            y: offset_wp.y + hero.y,
        };
        visible.insert(actual_wp);

        if map.get_tile(actual_wp).blocks_visibility() {
            shadow_line.add(shadow_a);
            maybe_shadow_b.map(|shadow_b| shadow_line.add(shadow_b));

            // in this case we're just done
            if shadow_line.is_full_shadow() {
                break;
            }
        }
    }

    visible
}

pub fn compute_viewsheds(
    mut visibility_events: EventWriter<VisibilityChangedEvent>,
    mut query: Query<(&mut Viewshed, &WorldPos)>,
    map: Res<Map>,
) {
    for (mut vs, wp) in query.iter_mut() {
        if !vs.dirty {
            continue;
        }

        vs.visible_tiles = refresh_area(*wp, vs.range as f32, &*map);

        vs.dirty = false;

        // TODO perf: in theory we only need to send this for the player?
        visibility_events.send(VisibilityChangedEvent);
    }
}

pub fn update_map_visibility(
    mut visibility_events: EventReader<VisibilityChangedEvent>,
    mut map_changed_events: EventWriter<MapChangedEvent>,
    query: Query<(&Viewshed, &Player)>,
    mut map: ResMut<Map>,
) {
    // Don't care about the details of the event, just that it occurred; we aren't doing "smart" updates
    if visibility_events.iter().next().is_none() {
        return;
    }

    for (vs, _) in query.iter() {
        map.set_visible_exact(&vs.visible_tiles);

        // the map "changed" so we need to recompute the visual tiles and stuff
        map_changed_events.send(MapChangedEvent);
    }
}
