# DTS Developer Challenge Submission

This repository contains my submission for the [DTS Developer Technical Test](https://github.com/hmcts/dts-developer-challenge).

## Usage

To run the project, you will need:

- An OCI/[Docker](https://www.docker.com/) compatible runtime; [docker engine](https://docs.docker.com/engine/) or [podman](https://podman.io/) are recommended.
- [`just`](https://github.com/casey/just).
  *Optional - manual commands are provided below*.
- A [Rust](https://www.rust-lang.org/) toolchain for rust `1.86` (this project's MSRV).
  [`rustup`](https://rustup.rs/) is recommended way to obtain this.
  *Optional - this is not required unless you wish to run the test suite or debug the backend locally*.

### Automatic

For convenience, this repository provides command-running shortcuts using `just`.
The application can be served at [`localhost:8080`](http://localhost:8080) with `just serve`.

### Manual

If `just` is not available or desirable, then the user should follow these steps:

1. Generate or otherwise create a suitable database password and store it in the root of this repo at `db_password.txt`.
2. Run the application with `docker compose up`.
3. Navigate to [the default frontend endpoint](http://localhost:8080) and enjoy!

## Technical Requirements

### Backend API

The backend should be able to:

 - Create a task with the following properties:
   - Title
   - Description (optional field)
   - Status
   - Due date/time
 - Retrieve a task by ID
 - Retrieve all tasks
 - Update the status of a task
 - Delete a task

### Frontend Application

The frontend should be able to:

- Create, view, update, and delete tasks
- Display tasks in a user-friendly interface

### General

- Implement unit tests
- Store data in a database
- Include validation and error handling
- Document API endpoints
