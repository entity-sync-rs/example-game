use legion::prelude::*;
use shared::components::Position;

use legion_sync::{components::UuidComponent, resources::ReceiveBufferResource};
use log::debug;
use track::{serialisation::bincode::Bincode, Apply};

pub fn read_received_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("read_received_system")
        .write_resource::<ReceiveBufferResource>()
        .with_query(<(legion::prelude::Write<Position>, Read<UuidComponent>)>::query())
        .build(|command_buffer, mut world, buffer, query| {
            for packet in buffer.drain() {
                match packet.event() {
                    legion_sync::Event::Inserted(_data) => {
                        command_buffer.insert(
                            (),
                            vec![(
                                Position { x: 0, y: 0 },
                                UuidComponent::from(packet.identifier().clone()),
                            )],
                        );

                        debug!("Inserted entity {:?}", packet.identifier());
                    }
                    legion_sync::Event::Modified(data) => {
                        for (mut pos, uuid) in query.iter_mut(&mut world) {
                            if uuid.uuid() == *packet.identifier() {
                                Apply::apply_to(&mut *pos, &data, Bincode);
                                break;
                            }
                        }

                        debug!("Modified entity {:?}", packet.identifier());
                    }
                    legion_sync::Event::Removed => {
                        debug!("Removed entity {:?}", packet.identifier());
                    }
                }
            }
        })
}