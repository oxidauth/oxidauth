#[derive(Clone, Debug)]
pub enum LoadingState<T> {
    Pending,
    Loading,
    Loaded(T),
    Error(String),
}

#[derive(Clone, Debug)]
pub enum SaveState {
    Idle,
    Saving,
    Saved,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum DeleteState {
    Idle,
    Deleting,
    Deleted,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum CreateState {
    Idle,
    Creating,
    Created(uuid::Uuid),
    Error(String),
}
