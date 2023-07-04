import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { path, event } from "@tauri-apps/api";
import { readTextFile } from "@tauri-apps/api/fs";

function App() {
  const [data, setData] = useState([]);
  const [tracking, setTracking] = useState(false);

  // make a function that reads a json file from /Documents/Traco/ and returns the data
  async function readJsonFile() {
    //  get lcoation of /Documents/Traco/
    const dir = await path.join(
      await path.homeDir(),
      "OneDrive",
      "Documents",
      "Traco"
    );

    // get the file name
    const fileName = "data.json";

    // read the file

    const data = await readTextFile(await path.join(dir, fileName));

    // parse the data
    const parsedData: any = JSON.parse(data);

    const formatedData: any = Object.keys(parsedData).map((key: any) => {
      let process = key.split("-");
      let app = process.pop();
      let task = process.join("-");

      // get time from epoch
      let date = new Date(
        parseInt(parsedData[key].start_time) * 1000
      ).toLocaleString("en-US", {
        hour12: true,
        hour: "numeric",
        minute: "numeric",
      });

      // convert duration to minutes
      let duration = parseInt(parsedData[key].duration);
      let minutes = Math.floor(duration / 60);
      let seconds = duration - minutes * 60;

      return {
        app: app,
        task: task,
        start_time: date,
        duration: `${minutes}m ${seconds}s`,
      };
    });

    setData(formatedData);

    // set the data
  }

  useEffect(() => {
    invoke("start_tracking")
      .then((res) => {
        console.log(res);
        setTracking(true);
      })
      .catch((err) => {
        console.log(err);
      });
    // listen for the event
    event.listen("tracking", (data) => {
      console.log(data);
      readJsonFile();
    });
    event.listen("stop_tracking", (data) => {
      setTracking(false);
    });
  }, []);

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
        }}
      >
        <span>Welcome Samarpit!</span>
        <span>Tracking: {tracking ? "Yes" : "No"} </span>
      </div>

      <button
        onClick={() => {
          readJsonFile();
        }}
      >
        Get Data
      </button>

      {tracking ? (
        <button
          onClick={() => {
            invoke("stop_tracking")
              .then((res) => {
                console.log(res);
              })
              .catch((err) => {
                console.log(err);
              });
          }}
        >
          Stop
        </button>
      ) : (
        <button
          onClick={() => {
            invoke("start_tracking")
              .then((res) => {
                console.log(res);
                setTracking(true);
              })
              .catch((err) => {
                console.log(err);
              });
          }}
        >
          Start
        </button>
      )}

      <div className="main-container">
        <table
          style={{
            border: "1px solid cyan",
            borderCollapse: "collapse",
            width: "70%",
            textAlign: "left",
          }}
        >
          <tr>
            <th>App</th>
            <th>Task</th>
            <th>Start Time</th>
            <th>Duration</th>
          </tr>
          {data?.map((item: any) => {
            return (
              <tr>
                <td>{item.app}</td>
                <td>{item.task}</td>
                <td>{item.start_time}</td>
                <td>{item.duration}</td>
              </tr>
            );
          })}
        </table>
        <div className="main-time">
          Total Time:{" "}
          {data?.reduce((acc: number, item: any) => {
            return acc + parseInt(item.duration.split(" ")[0]);
          }, 0)}
          m
        </div>
      </div>
    </div>
  );
}
export default App;
