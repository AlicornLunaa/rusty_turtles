use super::*;

#[test]
fn test_add_pop() {
    let mut scheduler = JobScheduler::new();
    scheduler.add_job(JobAction::Build, 10);
    scheduler.add_job(JobAction::Mine, 5);
    assert_eq!(scheduler.queue_size(), 2);
    assert_eq!(scheduler.pop_job(), Some(JobAction::Mine));
    assert_eq!(scheduler.queue_size(), 1);

    scheduler.add_job(JobAction::Mine, 15);

    assert_eq!(scheduler.pop_job(), Some(JobAction::Build));
    assert_eq!(scheduler.pop_job(), Some(JobAction::Mine));
    assert_eq!(scheduler.queue_size(), 0);

    assert_eq!(scheduler.pop_job(), None);
}