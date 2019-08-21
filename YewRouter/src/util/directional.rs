//! By using senders and receivers, components can set up message basses between themselves.
//! One or more components can create senders, and one or more components can create receivers,
//! and as long as they share the same type `T`, the message will be routed to the desired outputs.
//!
//! This provides the ability to avoid the "callback hell" incurred when a component needs to affect
//! the state of another component that is far away in the component hierarchy.

use yew::prelude::*;
use yew::prelude::worker::*;
use std::collections::HashSet;


/// A NewType to prevent users from subverting the pattern and directly sending messages to receivers.
#[derive(Serialize, Deserialize)]
pub struct DirectionalTransport<T>(T);

impl <'a, T> Transferable for DirectionalTransport<T>
    where T: Transferable {}

/// Can't implement Transferable for (), so use a personally defined Void type.
#[derive(Serialize, Deserialize)]
pub struct Void;
impl Transferable for Void {}



pub struct Sender<T>(Box<dyn Bridge<SenderImpl<T>>>) where T: Transferable + Clone + 'static;

impl <T> Sender<T> where T: Transferable + Clone + 'static {
    pub fn create<U>(link: &mut ComponentLink<U>) -> Self
        where
            U: Component,
            U::Message: Default, // TODO It would be nice if this constraint could be removed
            U: Renderable<U>
    {
        // It doesn't matter what the output of default() is, this callback will NEVER be called,
        // and so a message will never make it to the component's update method.
        let callback = link.send_back(|_| U::Message::default() );

        Sender (
            SenderImpl::<T>::bridge(callback)
        )
    }

    pub fn new(callback: Callback<Void> ) -> Self
    {
        Sender(SenderImpl::<T>::bridge(callback))
    }

    pub fn send(&mut self, msg: T) {
        self.0.send(msg)
    }
}

/// The sender will only accept inputs and will forward them to receivers of the same type.
/// The sender will never emit any outputs.
/// This creates a unidirectional message bus.
struct SenderImpl<T> where T: Transferable + Clone + 'static
{
    receiver: Box<dyn Bridge<ReceiverImpl<T>>>
}


impl<T> Agent for SenderImpl<T> where T: Transferable + Clone  + 'static
{
    type Reach = Context;
    type Message = ();
    type Input = T;
    type Output = Void; // TODO when the never_type stabilizes, I would like to replace this with `!`, yew will also have to impl Transferable for it

    fn create(link: AgentLink<Self>) -> Self {
        // This callback will never be called because the sender will be removed from its subscription list.
        let callback = link.send_back(|_| ());
        let receiver = ReceiverImpl::<T>::bridge(callback);

        SenderImpl {
            receiver
        }
    }

    fn update(&mut self, _msg: Self::Message) {
    }

    fn handle(&mut self, msg: Self::Input, _who: HandlerId) {
        self.receiver.send(DirectionalTransport(msg))
    }
}




pub struct Receiver<T>(Box<dyn Bridge<ReceiverImpl<T>>>) where T: Transferable + Clone + 'static;

impl <T> Receiver<T> where T: Transferable + Clone + 'static {
    /// Creates a Receiver of the same channel type as its component.
    /// If the channel type is different than the component's message type `new()` should be used instead.
    pub fn create<U>(link: &mut ComponentLink<U> ) -> Self
        where
            U: Component<Message=T> + Renderable<U>
    {
        let callback = link.send_back(|x| x);
        Receiver(
            ReceiverImpl::<<U as Component>::Message>::bridge(callback)
        )
    }
    /// Creates a Receiver using a callback that can map its input to the Message of the component.
    /// This should be used when the component's message type is different than its message type.
    pub fn new(callback: Callback<T> ) -> Self
    {
        Receiver (
            ReceiverImpl::<T>::bridge(callback)
        )
    }
}




struct ReceiverImpl<T> where T: Transferable + Clone + 'static
{
    /// The link that is used to broadcast to subscribers.
    link: AgentLink<ReceiverImpl<T>>,
    /// A list of all entities connected to the router.
    /// When a route changes, either initiated by the browser or by the app,
    /// the route change will be broadcast to all listening entities.
    subscribers: HashSet<HandlerId>,
}

impl<T> Agent for ReceiverImpl<T> where T: Transferable + Clone + 'static
{
    type Reach = Context;
    type Message = ();
    type Input = DirectionalTransport<T>;
    type Output = T;

    fn create(link: AgentLink<Self>) -> Self {
        ReceiverImpl {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle(&mut self, msg: Self::Input, who: HandlerId) {
        // Ensure that the sender isn't notified when broadcasting.
        self.subscribers.remove(&who);
        // Broadcast to all subscribers
        for sub in self.subscribers.iter() {
            self.link.response(*sub, msg.0.clone());
        }
    }
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}