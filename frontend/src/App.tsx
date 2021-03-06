import React, { Component } from 'react';
import {
  BrowserRouter as Router,
  Switch,
  Route,
} from "react-router-dom";
import Header from './Header';
import SignUp from './SignUp';
import LogIn from './LogIn';
import { GET_USER, rejectedPromiseHandler, SessionInfo } from './api';
import ApiGen from './ApiGen';
import EditUser from './EditUser';
import Games, { UrlGame } from './Games';
import Page from './Page';

interface AppState {
  session: SessionInfo;
}

export default class App extends Component<{}, AppState> {
  state = {
    session: { logged_in: false, has_api_key: false, username: "", display_name: "", id: -1, is_admin: false }
  }

  update_session = () => {
    fetch(GET_USER, { 
      method: 'GET',
      credentials: 'include',
     }).then((resp) => resp.json()).then(json => {
      if(json.error === undefined) {
        this.setState({session: {
          logged_in: true,
          has_api_key: json.has_api_key,
          username: json.username,
          display_name: json.display_name,
          id: json.id,
          is_admin: json.is_admin,
        }});
      } else {
        this.setState({session: {
          logged_in: false,
          has_api_key: false,
          username: "",
          display_name: "",
          id: -1,
          is_admin: false,
        }});
      }
    }).catch(rejectedPromiseHandler);
  };

  componentDidMount() {
    this.update_session();
  }

  render() {
    return (
      <Router>
        <div className="AppContainer">
          <Header session={this.state.session} session_change_callback={this.update_session} />
          <Switch>
            <Route exact path="/">
              <Games session={this.state.session}/>
            </Route>
            <Route exact path="/login">
              <LogIn callback={this.update_session} />
            </Route>
            <Route exact path="/signup">
              <SignUp />
            </Route>
            <Route exact path="/gen_api">
              <ApiGen session={this.state.session} session_change_callback={this.update_session}/>
            </Route>
            <Route exact path="/user_edit">
              <EditUser session={this.state.session} session_change_callback={this.update_session} />
            </Route>
            <Route path="/game/:game_id">
              <UrlGame session={this.state.session} />
            </Route>
            <Route path="/new_page">
              <Page session={this.state.session} newPage={true}></Page>
            </Route>
            <Route path="/pages/*">
              <Page session={this.state.session} newPage={false}></Page>
            </Route>
            <Route path="/">
              <NotFound />
            </Route>
          </Switch>
        </div>
      </Router>
    );
  }
}

export function NotFound(props: {}) {
  return (
    <p style={{textAlign: "center", fontSize: "2rem"}}>Page Not Found</p>
  );
}
