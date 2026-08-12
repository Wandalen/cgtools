//! Tests for the `events` module — event bus subscribe/publish/process lifecycle,
//! priorities, consumption, unsubscription, batching, statistics, common event
//! types, and utility listeners, driven purely through the public surface.
//!
//! Relocated from `src/events.rs` by task 072 (bodies verbatim; the fn-local
//! `use common_events::*;` was rewritten crate-qualified, since after relocation
//! the bare path no longer resolves against the parent module).

#![ cfg( feature = "enabled" ) ]


use tiles_tools::events::*;
use tiles_tools::coordinates::square::{Coordinate as SquareCoord, FourConnected};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct TestEvent {
  id: u32,
  message: String,
}

#[test]
fn test_event_bus_creation() {
  let bus = EventBus::new();
  assert_eq!(bus.channel_count(), 0);
  assert_eq!(bus.total_pending_count(), 0);
}

#[test]
fn test_subscribe_and_publish() {
  let mut bus = EventBus::new();
  let received = Arc::new(Mutex::new(Vec::new()));
  let received_clone = received.clone();

  bus.subscribe(move |event: &TestEvent| {
    received_clone.lock().unwrap().push(event.clone());
    EventResult::Continue
  });

  let event = TestEvent {
    id: 1,
    message: "test".to_string(),
  };

  bus.publish(event.clone());
  assert_eq!(bus.pending_count::<TestEvent>(), 1);

  bus.events_process();
  assert_eq!(bus.pending_count::<TestEvent>(), 0);

  let received_events = received.lock().unwrap();
  assert_eq!(received_events.len(), 1);
  assert_eq!(received_events[0].id, 1);
  assert_eq!(received_events[0].message, "test");
}

#[test]
fn test_event_priorities() {
  let mut bus = EventBus::new();
  let execution_order = Arc::new(Mutex::new(Vec::new()));

  // Add listeners in reverse priority order
  let order1 = execution_order.clone();
  bus.subscribe_with_priority(move |_: &TestEvent| {
    order1.lock().unwrap().push("low");
    EventResult::Continue
  }, EventPriority::Low);

  let order2 = execution_order.clone();
  bus.subscribe_with_priority(move |_: &TestEvent| {
    order2.lock().unwrap().push("critical");
    EventResult::Continue
  }, EventPriority::Critical);

  let order3 = execution_order.clone();
  bus.subscribe_with_priority(move |_: &TestEvent| {
    order3.lock().unwrap().push("normal");
    EventResult::Continue
  }, EventPriority::Normal);

  bus.publish(TestEvent { id: 1, message: "test".to_string() });
  bus.events_process();

  let order = execution_order.lock().unwrap();
  assert_eq!(*order, vec!["critical", "normal", "low"]);
}

#[test]
fn test_event_consumption() {
  let mut bus = EventBus::new();
  let received = Arc::new(Mutex::new(Vec::new()));

  // First listener consumes the event
  bus.subscribe(|_: &TestEvent| EventResult::Consume);

  // Second listener should never receive the event
  let received_clone = received.clone();
  bus.subscribe(move |event: &TestEvent| {
    received_clone.lock().unwrap().push(event.clone());
    EventResult::Continue
  });

  bus.publish(TestEvent { id: 1, message: "test".to_string() });
  bus.events_process();

  let received_events = received.lock().unwrap();
  assert_eq!(received_events.len(), 0); // Event was consumed before reaching second listener
}

#[test]
fn test_unsubscribe() {
  let mut bus = EventBus::new();
  let received = Arc::new(Mutex::new(0));
  let received_clone = received.clone();

  let listener_id = bus.subscribe(move |_: &TestEvent| {
    *received_clone.lock().unwrap() += 1;
    EventResult::Continue
  });

  // Publish and process first event
  bus.publish(TestEvent { id: 1, message: "test1".to_string() });
  bus.events_process();
  assert_eq!(*received.lock().unwrap(), 1);

  // Unsubscribe and publish second event
  assert!(bus.unsubscribe::<TestEvent>(listener_id));
  bus.publish(TestEvent { id: 2, message: "test2".to_string() });
  bus.events_process();
  assert_eq!(*received.lock().unwrap(), 1); // Should still be 1
}

#[test]
fn test_auto_unsubscribe() {
  let mut bus = EventBus::new();
  let call_count = Arc::new(Mutex::new(0));
  let counter_clone = call_count.clone();

  bus.subscribe(move |_: &TestEvent| {
    let mut count = counter_clone.lock().unwrap();
    *count += 1;
    if *count >= 2 {
      EventResult::Unsubscribe
    } else {
      EventResult::Continue
    }
  });

  // First event - listener remains
  bus.publish(TestEvent { id: 1, message: "test1".to_string() });
  bus.events_process();
  assert_eq!(bus.subscriber_count::<TestEvent>(), 1);

  // Second event - listener unsubscribes
  bus.publish(TestEvent { id: 2, message: "test2".to_string() });
  bus.events_process();
  assert_eq!(bus.subscriber_count::<TestEvent>(), 0);
}

#[test]
fn test_batch_publishing() {
  let mut bus = EventBus::new();
  let received = Arc::new(Mutex::new(Vec::new()));
  let received_clone = received.clone();

  bus.subscribe(move |event: &TestEvent| {
    received_clone.lock().unwrap().push(event.id);
    EventResult::Continue
  });

  let events = vec![
    TestEvent { id: 1, message: "test1".to_string() },
    TestEvent { id: 2, message: "test2".to_string() },
    TestEvent { id: 3, message: "test3".to_string() },
  ];

  bus.batch_publish(events);
  bus.events_process();

  let received_ids = received.lock().unwrap();
  assert_eq!(*received_ids, vec![1, 2, 3]);
}

#[test]
fn test_statistics() {
  let mut bus = EventBus::new();
  bus.subscribe(|_: &TestEvent| EventResult::Continue);

  assert_eq!(bus.statistics().events_published, 0);
  assert_eq!(bus.statistics().events_processed, 0);

  bus.publish(TestEvent { id: 1, message: "test".to_string() });
  assert_eq!(bus.statistics().events_published, 1);
  assert_eq!(bus.statistics().events_processed, 0);

  bus.events_process();
  assert_eq!(bus.statistics().events_processed, 1);
  assert_eq!(bus.statistics().process_cycles, 1);
}

#[test]
fn test_common_events() {
  use tiles_tools::events::common_events::*;

  let mut bus = EventBus::new();
  let moves = Arc::new(Mutex::new(Vec::new()));
  let moves_clone = moves.clone();

  bus.subscribe(move |event: &EntityMoved<SquareCoord<FourConnected>>| {
    moves_clone.lock().unwrap().push((event.entity_id, event.from, event.to));
    EventResult::Continue
  });

  bus.publish(EntityMoved {
    entity_id: 42,
    from: SquareCoord::<FourConnected>::new(1, 1),
    to: SquareCoord::<FourConnected>::new(2, 1),
    movement_type: MovementType::Walk,
  });

  bus.events_process();

  let recorded_moves = moves.lock().unwrap();
  assert_eq!(recorded_moves.len(), 1);
  assert_eq!(recorded_moves[0].0, 42);
}

#[test]
fn test_utility_functions() {
  let mut bus = EventBus::new();

  // Test counting listener
  let (listener, counter) = counting_listener::<TestEvent>();
  bus.subscribe(listener);

  bus.publish(TestEvent { id: 1, message: "test1".to_string() });
  bus.publish(TestEvent { id: 2, message: "test2".to_string() });
  bus.events_process();

  assert_eq!(*counter.lock().unwrap(), 2);
}

#[derive(Debug, Clone)]
struct EventA { value: i32 }

#[derive(Debug, Clone)]
struct EventB { text: String }

#[test]
fn test_multiple_event_types() {
  let mut bus = EventBus::new();

  let received_a = Arc::new(Mutex::new(Vec::new()));
  let received_b = Arc::new(Mutex::new(Vec::new()));

  let a_clone = received_a.clone();
  bus.subscribe(move |event: &EventA| {
    a_clone.lock().unwrap().push(event.value);
    EventResult::Continue
  });

  let b_clone = received_b.clone();
  bus.subscribe(move |event: &EventB| {
    b_clone.lock().unwrap().push(event.text.clone());
    EventResult::Continue
  });

  bus.publish(EventA { value: 42 });
  bus.publish(EventB { text: "hello".to_string() });
  bus.events_process();

  assert_eq!(*received_a.lock().unwrap(), vec![42]);
  assert_eq!(*received_b.lock().unwrap(), vec!["hello".to_string()]);
  assert_eq!(bus.channel_count(), 2);
}
