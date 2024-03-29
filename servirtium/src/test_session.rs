use crate::{error::Error, runner, ServirtiumConfiguration, ServirtiumMode, ServirtiumServer};
use lazy_static::lazy_static;
use std::sync::{Arc, Condvar, Mutex};

lazy_static! {
    static ref TEST_SESSION: TestSession = TestSession::new();
}

pub struct TestSession {
    lock: Arc<(Mutex<bool>, Condvar)>,
    error: Mutex<Option<Error>>,
}

impl TestSession {
    fn new() -> Self {
        Self {
            lock: Arc::new((Mutex::new(false), Condvar::new())),
            error: Mutex::new(None),
        }
    }

    pub(crate) fn set_error(error: Error) {
        *TEST_SESSION.error.lock().unwrap() = Some(error);
    }

    pub fn before_test(configuration: ServirtiumConfiguration) {
        TEST_SESSION.enter_test();
        runner::start_once();

        let mut server = ServirtiumServer::instance();

        server.configuration = Some(configuration);
        server.release_instance();
    }

    pub fn after_test() -> Result<(), Error> {
        let mut instance = ServirtiumServer::instance();

        let func = || {
            if let Some(err) = TEST_SESSION.error.lock().unwrap().take() {
                return Err(err);
            }

            let config = instance.configuration.as_ref().unwrap();
            let interaction_manager = config.interaction_manager().clone();

            if config.interaction_mode() == ServirtiumMode::Record {
                if !instance.interactions.is_empty() {
                    interaction_manager
                        .save_interactions(&instance.interactions)
                        .map_err(|e| Error::InteractionManager(e))?;
                }

                if config.fail_if_markdown_changed() {
                    interaction_manager
                        .check_data_unchanged(&instance.interactions)
                        .map_err(|e| Error::InteractionManager(e))?;
                }
            }

            Ok(())
        };

        let result = func();

        instance.reset();
        instance.release_instance();

        TEST_SESSION.exit_test();

        result
    }

    fn enter_test(&self) {
        let (lock, cond) = &*self.lock.clone();
        let mut is_test_running = cond
            .wait_while(lock.lock().unwrap(), |is_test_running| *is_test_running)
            .unwrap();
        *is_test_running = true;
    }

    fn exit_test(&self) {
        let (lock, cond) = &*self.lock.clone();
        let mut is_test_running = lock.lock().unwrap();
        *is_test_running = false;

        cond.notify_one();
    }
}

impl Default for TestSession {
    fn default() -> Self {
        Self::new()
    }
}
