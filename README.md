# RustNotesApi 📝🔒

A robust and secure API for personal note management, built with **Rust** and focused on privacy and performance.

## 🚀 Overview

**RustNotesApi** is the backend for a notes application that prioritizes security. It utilizes a modern architecture based on **Actix Web** and **SeaORM**, incorporating custom encryption layers and secure session management via key exchange.

The corresponding Android client for this API can be found here: [AndroidNotesExample](https://github.com/alexistamher/AndroidNotesExample)

## ✨ Main Features

-   **User Management:** Registration, login, and user profiles.
-   **Notes and Books:** Organization of notes by "books" (categories/notebooks).
-   **Advanced Security:**
    -   Diffie-Hellman key exchange to establish shared secrets.
    -   Custom encryption middleware (`CryptoMiddleware`) to protect communications.
    -   Authentication based on **JWT** (JSON Web Tokens).
-   **Database:** Efficient persistence using **SQLite** and the **SeaORM** ORM.
-   **Cache:** In-memory session management system for fast validation.

## 🛠️ Technologies Used

-   **Language:** Rust (2024 Edition)
-   **Web Framework:** [Actix Web](https://actix.rs/)
-   **ORM:** [SeaORM](https://www.sea-ql.org/SeaORM/)
-   **Database:** SQLite
-   **Cryptography:** `rust-cipher-lib` (custom library)
-   **Others:** JWT, Chrono, Uuid, Dotenv.

## 📁 Project Structure

```text
src/
├── data/       # Repositories and database models (SeaORM)
├── domain/     # Business models and request/response structures
├── routes/     # API controllers and Middlewares
├── util/       # Utilities (JWT, Cache, Sessions, Configuration)
└── main.rs     # Entry point and server configuration
```

## ⚙️ Configuration

To run the project locally, ensure you have Rust installed and create a `.env` file in the root directory with the following variables:

```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=sqlite://db.sqlite?mode=rwc
SERVER_SECRET=your_jwt_secret
SERVER_LIFETIME=3600
```

## 🚀 How to Run

1. Clone the repository.
2. Set up your `.env` file.
3. Run migrations (if using SeaORM CLI) or ensure `db.sqlite` is available.
4. Start the server:
   ```bash
   cargo run
   ```

## 🔒 Security Flow

The system uses a **key exchange** process (`/session/exchange`) before sensitive operations. This allows establishing an encrypted connection where the client and server share a secret without transmitting it directly, ensuring an additional layer of protection over HTTPS.

