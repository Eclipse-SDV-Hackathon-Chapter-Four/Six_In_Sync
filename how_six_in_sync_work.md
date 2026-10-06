
# Development Process

We use a lightweight Kanban-based development process to plan and track the project throughout the three-day hackathon. All tasks
are managed in a shared GitHub Project, providing the team with a single overview of responsibilities, priorities, and progress.
Tasks are organized by status, priority, technical area, and hackathon day, with the workflow progressing from Backlog and Todo
to In Progress, Review/Test, and finally Done.
The three-day schedule is divided into clear iterations: Day 1 – Foundation, focusing on the development environment, core
Guardian logic, and basic sensor simulation; Day 2 – Integration, focusing on communication and the integration of sensors
Guardian, and actuation; and Day 3 – Demo & Stabilization, focusing on end-to-end testing, debugging, documentation, and
preparing the final demonstration. Tasks are assigned to team members and kept small enough to allow progress to be tracked
continuously. This lightweight structure gives the team a clear overview of what needs to be done while remaining flexible enough
to react quickly to technical challenges and changing priorities during the hackathon.

# Quality Control

To ensure a consistent level of quality throughout the project, we will use a lightweight but structured quality process. Core
functionality, especially the Guardian state machine, will be covered by unit tests to verify the expected behavior for different
child-presence and temperature scenarios. In addition, integration and end-to-end tests will be used to validate the
communication between the simulated sensors, Guardian logic, and actuation layer.
All code changes will be developed in separate branches and reviewed through pull requests before being merged into the main
branch. This provides a second pair of eyes for important changes and helps identify bugs or design issues early. We will also
keep the project documentation up to date, including the README and relevant technical decisions, so that the implementation and
architecture remain understandable for the entire team.
A task is considered complete only when the implementation works as intended, relevant tests have been executed successfully, and
the corresponding code has been reviewed. This approach allows us to maintain code quality and traceability while keeping the
development process efficient and suitable for the limited three-day hackathon timeframe.

# Team Communication

To ensure effective communication under the time constraints of the three-day hackathon, we will use a simple and direct
communication structure. The shared GitHub Project serves as the central source of truth for tasks, responsibilities, priorities,
and progress, while a dedicated team chat will be used for quick day-to-day communication.
We will have short team syncs to align on current progress, upcoming tasks, and potential blockers. Team members are
encouraged to communicate blockers as early as possible rather than spending excessive time on isolated problems. For complex
technical issues, we will use short focused discussions with the relevant team members to reach a solution quickly.
To avoid communication overhead, decisions that affect the overall architecture or multiple work areas will be discussed with the
team, while implementation decisions within an assigned task can be made directly by the responsible developer. This structure
allows us to stay aligned, resolve issues quickly, and keep the majority of our time focused on implementation and integration.