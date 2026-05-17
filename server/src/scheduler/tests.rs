use super::*;

#[tokio::test]
async fn test_add_pop() {
    let scheduler = TaskScheduler::new();
    scheduler.add_task(TaskAction::Place { x: 0, y: 0, z: 0, block: "minecraft:stone".into() }, 10).await;
    scheduler.add_task(TaskAction::Break { x: 0, y: 0, z: 0 }, 5).await;
    assert_eq!(scheduler.len().await, 2);
    assert_eq!(scheduler.pop_task().await, Some(TaskAction::Break { x: 0, y: 0, z: 0 }));
    assert_eq!(scheduler.len().await, 1);

    scheduler.add_task(TaskAction::Break { x: 0, y: 0, z: 0 }, 15).await;

    assert_eq!(scheduler.pop_task().await, Some(TaskAction::Place { x: 0, y: 0, z: 0, block: "minecraft:stone".into() }));
    assert_eq!(scheduler.pop_task().await, Some(TaskAction::Break { x: 0, y: 0, z: 0 }));
    assert_eq!(scheduler.len().await, 0);

    assert_eq!(scheduler.pop_task().await, None);
}

#[tokio::test]
async fn test_system(){

}