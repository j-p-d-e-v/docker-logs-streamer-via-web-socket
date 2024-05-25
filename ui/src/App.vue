
<template>
 <v-sheet color="#0d1117" elevation="0" style="height:100vh">
            <v-tabs
              slider-color="#f78166"
              v-model="tab"
              fixed-tabs
            >
              <v-tab :value="'config'">Configuration</v-tab>
              <v-tab :value="'viewer'">Log Viewer</v-tab>              
          </v-tabs>
          <v-tabs-window v-model="tab" style="height:90vh;">                
            <v-tabs-window-item value="config">
              <div style="width:100%;padding:10px 20px 10px 20px;text-align:left;">
                <div class="form-input">
                  <v-text-field
                    hint="The container id or name displayed in docker ps."
                    label="Container Name"
                    v-model="container_name"
                    variant="outlined"
                  ></v-text-field>
                </div>
                <div class="form-input">
                  <v-text-field
                    hint="The session name of your choice and this will be used as filename your log."
                    label="Session ID"
                    v-model="session_id"
                    variant="outlined"
                  ></v-text-field>
                </div>
                <div class="form-input">
                  <v-text-field
                    type="number"
                    hint="The number of seconds the connection will disconnect if the client dont reply or do acknowledgement."
                    label="Timeout"
                    min="0"
                    v-model="timeout"
                    variant="outlined"
                  ></v-text-field>
                </div>
                <div class="form-input">
                  <v-text-field
                    type="number"
                    hint="Show logs since N number of minutes. Example: 30 minutes ago."
                    label="Since (In Minutes)"
                    min="0"
                    v-model="since_in_minutes"
                    variant="outlined"
                  ></v-text-field>
                </div>
                <div class="form-input">
                  <v-text-field
                    hint="The authentication key in order to listen to the container logs."
                    label="Stream Key"
                    v-model="stream_key"
                    variant="outlined"
                  ></v-text-field>
                </div>
                <div class="form-input">
                    <v-switch
                      v-model="save_logs"
                      label="Save Logs?"
                      color="red"
                      inset
                    ></v-switch>
                </div>
                
              </div>
            </v-tabs-window-item>              
            <v-tabs-window-item value="viewer">            
              <div id="logs">
                <pre v-for="(message,key) in data" :key="key">{{ message }}</pre>
              </div>
              <v-btn
                  v-if="connected" 
                  @click="onDisconnectWS"
                  class="text-none"
                  color="red"
                  size="large"
                  block
                  :rounded="0"
                >
                  Disconnect
              </v-btn>    
              <v-btn
                  :rounded="0"
                  v-if="!connected" 
                  @click="onConnectWS"
                  class="text-none"
                  color="blue"
                  size="large"
                  block
                >
                  Connect
              </v-btn>  
              <v-btn
                  v-if="can_download && save_logs"
                  :rounded="0"
                  @click="downloadLog"
                  class="text-none"
                  color="grey"
                  size="large"
                  block
                >
                  Download
              </v-btn> 
            </v-tabs-window-item>
          </v-tabs-window>
</v-sheet>
</template>

<style scoped>
#logs pre {
  font-size:13pt;
}
#logs{
  padding:10px;
  border-top:5px solid #283850;
  border-bottom:2px solid #283850;
  height:70vh;
  width:100%;
  overflow:auto;
  background-color:#000;
  color:#FFF;
  text-align: left;
}
</style>
<script>
export default{
  methods:{
    downloadLog(){
      let download_url = `http://${this.host}/logs/${this.session_id}.log`;
      window.open(download_url);
    },
    onConnectWS(){
      let base_url = "ws://"+this.host+"/watch";
      let params = [
        "container_name="+this.container_name,
        "session_id="+this.session_id,
        "timeout="+this.timeout,
        "stream_key="+this.stream_key,
        "save_logs="+this.save_logs,
        "since_in_minutes="+this.since_in_minutes,
      ].join("&");
      this.can_download = false;
      let url = `${base_url}?${params}`;
      console.log(url);
      this.ws = new WebSocket(url);
      this.ws.onopen = (e) => {
        console.info("Websocket Connection Established",e);
        this.connected = true;
      };
      this.ws.onmessage = (e) => {
        let data = e.data;
        let total = this.data.length;
        if(total >= 30) {
         this.data = this.data.slice(7,9); 
        }
        this.data.push(data);
        this.ws.send("ACK");
      };
      this.ws.onerror = (e) => {
        this.connected = false;
        this.can_download = true;
        console.error(e);
      };
      this.ws.onclose = (e) => {
        this.connected = false;
        this.can_download = true;
        console.log("Closing Websocket",e);
      };
    },
    onDisconnectWS(){
      this.can_download = true;
      this.ws.close();
    }
  },
  data(){
    return {
      host: "localhost:3090",
      tab: "config",
      container_name: "random_messages",
      session_id: "session-1576052f73f8",
      save_logs: false,
      since_in_minutes:0,
      timeout:30,
      stream_key: "thisisastreamkeyfortesting",
      data:[],
      connected:false,
      can_download: false,
      ws:null
    }
  }
}
</script>
