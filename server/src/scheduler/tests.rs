use super::*;

#[test]
fn test_add_pop() {
    let mut scheduler = TaskScheduler::new();
    scheduler.add_task(TaskAction::Place { x: 0, y: 0, z: 0, block: "minecraft:stone".into() }, 10);
    scheduler.add_task(TaskAction::Break { x: 0, y: 0, z: 0 }, 5);
    assert_eq!(scheduler.len(), 2);
    assert_eq!(scheduler.pop_task(), Some(TaskAction::Break { x: 0, y: 0, z: 0 }));
    assert_eq!(scheduler.len(), 1);

    scheduler.add_task(TaskAction::Break { x: 0, y: 0, z: 0 }, 15);

    assert_eq!(scheduler.pop_task(), Some(TaskAction::Place { x: 0, y: 0, z: 0, block: "minecraft:stone".into() }));
    assert_eq!(scheduler.pop_task(), Some(TaskAction::Break { x: 0, y: 0, z: 0 }));
    assert_eq!(scheduler.len(), 0);

    assert_eq!(scheduler.pop_task(), None);
}