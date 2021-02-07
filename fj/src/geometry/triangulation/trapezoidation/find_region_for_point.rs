use crate::geometry::triangulation::trapezoidation::{
    point::{self, Point},
    segment,
};

use super::{
    graph::{Graph, Node, X, Y},
    ids::Id,
};

/// Find the region that the given point is in
///
/// Returns `None` if a region can't be found because the point is already in
/// the graph.
pub fn find_region_for_point<Region>(
    point: &Point,
    graph: &Graph<X, Y, Region>,
) -> Option<Id> {
    let mut current_id = graph.source();

    loop {
        match graph.get(current_id) {
            Node::X(X {
                segment,
                left,
                right,
            }) => match segment.relation_from_point(point) {
                Some(segment::Relation::Left) => current_id = *left,
                Some(segment::Relation::Right) => current_id = *right,
                None => {
                    // I don't think I have to handle this case, as it can only
                    // happen if the tree is misshapen or we're getting invalid
                    // input, meaning some other code would have to be buggy.
                    // I'm not completely sure though, so please make up your
                    // own mind if you happen to hit this panic.
                    panic!(
                        "No relation from point ({:?}) to segment ({:?})",
                        point, segment
                    )
                }
            },
            Node::Y(Y {
                point: p,
                below,
                above,
            }) => match point.relation_to(p) {
                Some(point::Relation::Below) => current_id = *below,
                Some(point::Relation::Above) => current_id = *above,
                None => {
                    if p == point {
                        // Point already in graph.
                        return None;
                    }

                    // If we land here, the points have no relation to each
                    // other, but also aren't equal. Something shady must be
                    // happening, like NaN.
                    panic!("Invalid point: {:?}", point);
                }
            },
            Node::Sink(_) => return Some(current_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::triangulation::trapezoidation::{
        graph::{self, Node, X, Y},
        point::Point,
        segment::Segment,
    };

    use super::find_region_for_point;

    type Graph = graph::Graph<X, Y, Region>;

    #[derive(Debug, Default)]
    struct Region(u64);

    #[test]
    fn find_region_for_point_find_root_region_if_none_other_exist() {
        let graph = Graph::new();

        let region = find_region_for_point(&Point::new(0.0, 0.0), &graph);
        assert_eq!(region, Some(graph.source()));
    }

    #[test]
    fn find_region_for_point_determine_if_point_is_left_or_right_of_x_node() {
        let mut graph = Graph::new();

        let left = graph.insert_sink(Region(1));
        let right = graph.insert_sink(Region(2));

        let node = Node::X(X {
            segment: Segment::new(Point::new(1.0, 0.0), Point::new(1.0, 2.0))
                .unwrap(),
            left,
            right,
        });

        graph.replace(graph.source(), node);

        assert_eq!(
            find_region_for_point(&Point::new(0.0, 1.0), &graph),
            Some(left)
        );
        assert_eq!(
            find_region_for_point(&Point::new(2.0, 1.0), &graph),
            Some(right)
        );
    }

    #[test]
    fn find_region_for_point_determine_if_point_is_below_or_above_a_y_node() {
        let mut graph = Graph::new();

        let below = graph.insert_sink(Region(1));
        let above = graph.insert_sink(Region(2));

        let node = Node::Y(Y {
            point: Point::new(0.0, 1.0),
            below,
            above,
        });

        graph.replace(graph.source(), node);

        assert_eq!(
            find_region_for_point(&Point::new(0.0, 0.0), &graph),
            Some(below)
        );
        assert_eq!(
            find_region_for_point(&Point::new(0.0, 2.0), &graph),
            Some(above)
        );
    }

    #[test]
    fn find_region_for_point_return_id_of_point_if_already_present() {
        let mut graph = Graph::new();

        let below = graph.insert_sink(Region(1));
        let above = graph.insert_sink(Region(2));

        let point = Point::new(0.0, 1.0);
        let node = Node::Y(Y {
            point,
            below,
            above,
        });

        graph.replace(graph.source(), node);

        assert_eq!(find_region_for_point(&point, &graph), None);
    }
}
