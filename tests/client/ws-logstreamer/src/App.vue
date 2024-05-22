
<template>
  <div>
    <div id="logs" style="width:50%;float:left;margin-top:10px;">
      <pre v-for="(message,key) in data" :key="key">{{ message }}</pre>
    </div>
    <div style="width:40%;float:left;padding:10px 20px 10px 20px;text-align:left;">
      <div class="form-input">
        <label>Container Name</label>
        <input type="text" v-model="container_name" />
      </div>
      <div class="form-input">
        <label>Session ID</label>
        <input type="text" v-model="session_id" />
      </div>
      <div class="form-input">
        <label>Timeout</label>
        <input type="number" v-model="timeout" />
      </div>
      <div class="form-input">
        <label>Stream Key</label>
        <input type="text" v-model="stream_key" />
      </div>
      <div>
        <br/>
        <button v-if="!connected" @click="onConnectWS">Submit</button>
        <br/>
        <button v-if="connected" @click="onDisconnectWS">Disconnect</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
#logs pre {
  line-height: 8px;
  font-size:9pt;
}
#logs{
  padding:10px;
  border:1px solid #FFF;
  height:250px;
  width:100%;
  overflow:auto;
  background-color:#000;
  color:#FFF;
  text-align: left;
}
.form-input input {
  width:100%;
  padding:10px;
  margin-bottom:10px;
}
</style>
<script>
export default{
  methods:{
    onConnectWS(){
      let base_url = "ws://localhost:3090/watch";
      let params = [
        "container_name="+this.container_name,
        "session_id="+this.session_id,
        "timeout="+this.timeout,
        "stream_key="+this.stream_key,
        "save_logs="+this.save_logs
      ].join("&");
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
        if(total >= 10) {
         this.data = this.data.slice(7,9); 
        }
        this.data.push(data);
        this.ws.send("ACK");
      };
      this.ws.onerror = (e) => {
        this.connected = false;
        console.error(e);
      };
      this.ws.onclose = (e) => {
        this.connected = false;
        console.log("Closing Websocket",e);
      };
    },
    onDisconnectWS(){
      this.ws.close();
    }
  },
  data(){
    return {
      message:"Hello Tiktok",
      container_name: "1576052f73f8",
      session_id: "session-1576052f73f8",
      save_logs: true,
      timeout:30,
      stream_key: "thisisastreamkeyfortesting",
      data:[],
      connected:false,
      ws:null
    }
  }
}
</script>
