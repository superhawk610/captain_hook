use crate::job::{Job, Socket};
use crate::spawn;
use std::cell::RefCell;
use uuid::Uuid;
use ws::Sender;

pub struct Pool {
    jobs: RefCell<Vec<Job>>,
    sockets: RefCell<Vec<Socket>>,
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            jobs: RefCell::new(Vec::new()),
            sockets: RefCell::new(Vec::new()),
        }
    }

    pub fn register(&self, sender: Sender) -> String {
        let mut sockets = self.sockets.borrow_mut();
        let id = Uuid::new_v4().to_hyphenated().to_string();

        sockets.push(Socket::new(id.clone(), sender));

        id
    }

    pub fn spawn(&self, arg: &str, socket: &str) -> String {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let output = spawn::spawn(arg);

        let job_id = {
            let mut jobs = self.jobs.borrow_mut();

            // remove socket from the pool and place into the newly spawned job
            let idx = self.get_socket_idx(socket).unwrap();
            let mut sockets = self.sockets.borrow_mut();
            let socket = sockets.swap_remove(idx);

            let job = Job::new(id.clone(), output, socket);
            let job_id = job.get_id();
            job.run();
            jobs.push(job);

            job_id
        };

        self.remove_job(&job_id).expect("failed to remove job");

        id
    }

    pub fn assign_socket_to_job(&self, socket_id: &str, job_id: &str) -> Result<(), String> {
        // look for socket in each currently running job
        let jobs = self.jobs.borrow();
        for job in jobs.iter() {
            let mut sockets = job.sockets.borrow_mut();
            for (idx, job_socket) in sockets.iter().enumerate() {
                if job_socket.get_id() == socket_id {
                    // we've found it, remove this socket from its current job
                    let socket = sockets.swap_remove(idx);

                    // look for job
                    if let Ok(job_idx) = self.get_job_idx(job_id) {
                        let job = &jobs[job_idx];
                        let mut sockets = job.sockets.borrow_mut();

                        sockets.push(socket);

                        return Ok(());
                    };

                    return Err(format!("job {} couldn't be found", job_id));
                }
            }
        }

        Err(format!("socket {} couldn't be found", socket_id))
    }

    pub fn remove_job(&self, id: &str) -> Result<(), String> {
        match self.get_job_idx(id) {
            Ok(idx) => {
                let mut jobs = self.jobs.borrow_mut();

                // remove the job from the pool
                let job = jobs.remove(idx);

                // return all of the job's sockets to the pool for further use
                let mut job_sockets = job.sockets.into_inner();
                let mut sockets = self.sockets.borrow_mut();
                sockets.append(&mut job_sockets);

                Ok(())
            }
            Err(_) => Err(format!("no job matching id {} found to remove", id)),
        }
    }

    fn get_socket_idx(&self, id: &str) -> Result<usize, String> {
        let sockets = self.sockets.borrow();
        for (idx, socket) in sockets.iter().enumerate() {
            if socket.get_id() == id {
                return Ok(idx);
            }
        }

        Err(format!("no socket matching id {} found", id))
    }

    fn get_job_idx(&self, id: &str) -> Result<usize, String> {
        let jobs = self.jobs.borrow();
        for (idx, job) in jobs.iter().enumerate() {
            if job.get_id() == id {
                return Ok(idx);
            }
        }

        Err(format!("no job matching id {} found", id))
    }
}
