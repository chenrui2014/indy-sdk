use errors::common::CommonError;
use errors::sovrin::SovrinError;
use errors::pool::PoolError;

use services::pool::PoolService;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;


pub enum PoolCommand {
    Create(String, // name
           Option<String>, // config
           Box<Fn(Result<(), SovrinError>) + Send>),
    Delete(String, // name
           Box<Fn(Result<(), SovrinError>) + Send>),
    Open(String, // name
         Option<String>, // config
         Box<Fn(Result<i32, SovrinError>) + Send>),
    OpenAck(i32, // cmd id
            Result<i32 /* pool handle */, PoolError>),
    Close(i32, // pool handle
          Box<Fn(Result<(), SovrinError>) + Send>),
    CloseAck(i32,
             Result<(), PoolError>),
    Refresh(i32, // pool handle
            Box<Fn(Result<(), SovrinError>) + Send>),
    RefreshAck(i32,
               Result<(), PoolError>),
}

pub struct PoolCommandExecutor {
    pool_service: Rc<PoolService>,
    close_callbacks: RefCell<HashMap<i32, Box<Fn(Result<(), SovrinError>)>>>,
    refresh_callbacks: RefCell<HashMap<i32, Box<Fn(Result<(), SovrinError>)>>>,
    open_callbacks: RefCell<HashMap<i32, Box<Fn(Result<i32, SovrinError>)>>>,
}

impl PoolCommandExecutor {
    pub fn new(pool_service: Rc<PoolService>) -> PoolCommandExecutor {
        PoolCommandExecutor {
            pool_service: pool_service,
            close_callbacks: RefCell::new(HashMap::new()),
            refresh_callbacks: RefCell::new(HashMap::new()),
            open_callbacks: RefCell::new(HashMap::new()),
        }
    }

    #[allow(unused_variables)] /* FIXME */
    pub fn execute(&self, command: PoolCommand) {
        match command {
            PoolCommand::Create(name, config, cb) => {
                info!(target: "pool_command_executor", "Create command received");
                self.create(&name, config.as_ref().map(String::as_str), cb);
            }
            PoolCommand::Delete(name, cb) => {
                info!(target: "pool_command_executor", "Delete command received");
                self.delete(&name, cb);
            }
            PoolCommand::Open(name, config, cb) => {
                info!(target: "pool_command_executor", "Open command received");
                self.open(&name, config.as_ref().map(String::as_str), cb);
            }
            PoolCommand::OpenAck(handle, result) => {
                info!("OpenAck handle {:?}, result {:?}", handle, result);
                match self.open_callbacks.try_borrow_mut() {
                    Ok(mut cbs) => {
                        match cbs.remove(&handle) {
                            Some(cb) => cb(result.map_err(SovrinError::from)),
                            None => {
                                error!("Can't process PoolCommand::OpenAck for handle {} with result {:?} - appropriate callback not found!",
                                handle, result);
                            }
                        }
                    }
                    Err(err) => { error!("{:?}", err); }
                }
            }
            PoolCommand::Close(handle, cb) => {
                info!(target: "pool_command_executor", "Close command received");
                self.close(handle, cb);
            }
            PoolCommand::CloseAck(handle, result) => {
                info!(target: "pool_command_executor", "CloseAck command received");
                match self.close_callbacks.try_borrow_mut() {
                    Ok(mut cbs) => {
                        match cbs.remove(&handle) {
                            Some(cb) => cb(result.map_err(SovrinError::from)),
                            None => {
                                error!("Can't process PoolCommand::CloseAck for handle {} with result {:?} - appropriate callback not found!",
                                handle, result);
                            }
                        }
                    }
                    Err(err) => { error!("{:?}", err); }
                }
            }
            PoolCommand::Refresh(handle, cb) => {
                info!(target: "pool_command_executor", "Refresh command received");
                self.refresh(handle, cb);
            }
            PoolCommand::RefreshAck(handle, result) => {
                info!(target: "pool_command_executor", "RefreshAck command received");
                match self.refresh_callbacks.try_borrow_mut() {
                    Ok(mut cbs) => {
                        match cbs.remove(&handle) {
                            Some(cb) => cb(result.map_err(SovrinError::from)),
                            None => {
                                error!("Can't process PoolCommand::RefreshAck for handle {} with result {:?} - appropriate callback not found!",
                                handle, result);
                            }
                        }
                    }
                    Err(err) => { error!("{:?}", err); }
                }
            }
        };
    }

    fn create(&self, name: &str, config: Option<&str>, cb: Box<Fn(Result<(), SovrinError>) + Send>) {
        cb(self.pool_service.create(name, config).map_err(SovrinError::from))
    }

    fn delete(&self, name: &str, cb: Box<Fn(Result<(), SovrinError>) + Send>) {
        cb(self.pool_service.delete(name).map_err(SovrinError::from));
    }

    fn open(&self, name: &str, config: Option<&str>, cb: Box<Fn(Result<i32, SovrinError>) + Send>) {
        let result = self.pool_service.open(name, config)
            .map_err(|err| SovrinError::PoolError(err))
            .and_then(|handle| {
                match self.open_callbacks.try_borrow_mut() {
                    Ok(cbs) => Ok((cbs, handle)),
                    Err(err) => Err(SovrinError::PoolError(PoolError::from(CommonError::from(err)))),
                }
            });
        match result {
            Err(err) => { cb(Err(err)); }
            Ok((mut cbs, handle)) => { cbs.insert(handle, cb); /* TODO check if map contains same key */ }
        };
    }

    fn close(&self, handle: i32, cb: Box<Fn(Result<(), SovrinError>) + Send>) {
        let result = self.pool_service.close(handle)
            .map_err(From::from)
            .and_then(|handle| {
                match self.close_callbacks.try_borrow_mut() {
                    Ok(cbs) => Ok((cbs, handle)),
                    Err(err) => Err(SovrinError::PoolError(PoolError::from(CommonError::from(err))))
                }
            });
        match result {
            Err(err) => { cb(Err(err)); }
            Ok((mut cbs, handle)) => { cbs.insert(handle, cb); /* TODO check if map contains same key */ }
        };
    }

    fn refresh(&self, handle: i32, cb: Box<Fn(Result<(), SovrinError>) + Send>) {
        let result = self.pool_service.refresh(handle)
            .map_err(From::from)
            .and_then(|handle| {
                match self.refresh_callbacks.try_borrow_mut() {
                    Ok(cbs) => Ok((cbs, handle)),
                    Err(err) => Err(SovrinError::PoolError(PoolError::from(CommonError::from(err))))
                }
            });
        match result {
            Err(err) => { cb(Err(err)); }
            Ok((mut cbs, handle)) => { cbs.insert(handle, cb); /* TODO check if map contains same key */ }
        };
    }
}