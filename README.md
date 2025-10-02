<h2 align="center"> ━━━━━━━━━━  ❖  ━━━━━━━━━━ </h2>

<!-- BADGES -->
<div align="center">

[![stars](https://img.shields.io/github/stars/zelvios/simple-booking?color=C9CBFF&labelColor=1A1B26&style=for-the-badge)](https://github.com/zelvios/simple-booking)
[![Visitors](https://api.visitorbadge.io/api/visitors?path=https%3A%2F%2Fgithub.com%2Fzelvios%2Fsimple-booking&label=View&labelColor=%231a1b26&countColor=%23e0af68)](https://visitorbadge.io/status?path=https%3A%2F%2Fgithub.com%2Fzelvios%2Fsimple-booking)
[![license](https://img.shields.io/github/license/zelvios/simple-booking?color=FCA2AA&labelColor=1A1B26&style=for-the-badge)](https://github.com/zelvios/simple-booking/blob/main/LICENSE.md)

</div>

<h2></h2>

# simple-booking
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/Svelte-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)](https://svelte.dev/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-336791?style=for-the-badge&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

> [!IMPORTANT]
> This is a very early version. 
> Expect some parts to be experimental and subject to change. 
> Not everything may be fully configured or optimized yet.

## 🌿 <samp>About</samp>

<img src=".github/screenshots/design.png" alt="simple-booking Showcase" align="right" width="350px">


**simple-booking** is a full-stack booking system with a Tauri & Svelte frontend and a Rust backend using Actix-web, Diesel + PostgreSQL, JWT authentication, and Docker deployment.

Setup:
- **Backend:**
  - Framework: [`Actix-web`](https://actix.rs/)
  - ORM: [`Diesel`](https://diesel.rs/) (PostgreSQL support)
  - Database: [`PostgreSQL`](https://www.postgresql.org/)
  - Connection Pooling: [`R2D2`](https://docs.rs/r2d2/)
  - Authentication: JWT (`jsonwebtoken`)
  - Password hashing: [`argon2`](https://crates.io/crates/argon2)
  - Environment: [`dotenvy`](https://crates.io/crates/dotenvy)
  - Containerization: [`Docker`](https://www.docker.com/)

- **Frontend:**
  - Framework: [`Svelte`](https://svelte.dev/)
  - Desktop wrapper: [`Tauri`](https://tauri.app/)
