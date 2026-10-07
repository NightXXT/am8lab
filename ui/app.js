'use strict';
const $=id=>document.getElementById(id), invoke=(name,args={})=>window.__TAURI__.core.invoke(name,args);
const state={busy:false,connected:false,pending:false,snapshot:null,dirty:new Set(),smoke:false,lastError:null,view:"overview",applying:null,failedEffect:null};
const updates={info:null,busy:'',native:false,restricted:true,notice:'',noticeError:false};
const updateStatuses={
 idle:['Verifique quando quiser','A consulta só acontece ao clicar em Verificar atualizações.','↻'],
 available:['Nova versão disponível','Uma versão mais recente está disponível para Windows.','↓'],
 current:['Você está atualizado','Sua versão corresponde à mais recente publicada no GitHub.','✓'],
 ahead:['Versão local mais recente','A versão publicada no GitHub é anterior à instalada. Nenhuma atualização é necessária.','✓'],
 no_release:['Nenhuma versão publicada','Ainda não há uma release disponível neste repositório.','○'],
 unavailable:['Instalador indisponível','Há uma release, mas ela ainda não oferece um instalador compatível. Confira os detalhes em Releases.','○'],
 error:['Não foi possível verificar','Confira sua conexão e tente novamente. Você também pode abrir Releases no navegador.','!']
};
function updateDownloadReady(){return updates.info?.status==='available'&&typeof updates.info.latest_version==='string'&&typeof updates.info.download_url==='string'&&!!updates.info.download_url;}
function renderUpdates(){
 const info=updates.info,status=Object.hasOwn(updateStatuses,info?.status)?info.status:'idle',checking=updates.busy==='checking';
 const [title,hint,icon]=checking?['Verificando atualizações…','Consultando as versões publicadas no GitHub.','↻']:updateStatuses[status];
 const version=typeof info?.current_version==='string'?info.current_version:null,latest=typeof info?.latest_version==='string'?info.latest_version:null;
 $('updatesVersion').textContent=version?'Versão '+version:'AM8 Lab';$('updatesInstalled').textContent=version?'v'+version:'—';$('updatesLatest').textContent=latest?'v'+latest:status==='no_release'?'Sem release':'Não verificada';
 $('updatesSidebarStatus').textContent=checking?'Verificando…':status==='available'?'Nova versão '+latest:status==='current'?'Você está atualizado':status==='ahead'?'Versão local mais recente':status==='error'?'Tentar novamente':status==='no_release'?'Sem release publicada':status==='unavailable'?'Instalador indisponível':'Verifique quando quiser';
 $('updatesDot').hidden=status!=='available';$('updatesOpen').classList.toggle('has-update',status==='available');
 $('updatesStatePanel').dataset.status=checking?'checking':status;$('updatesStateTitle').textContent=title;$('updatesStateHint').textContent=hint;$('updatesStateIcon').textContent=icon;$('updatesStateIcon').classList.toggle('is-checking',checking);
 const releaseName=typeof info?.release_name==='string'?info.release_name.slice(0,256):'',notes=typeof info?.release_notes==='string'?info.release_notes.slice(0,4000):'';
 $('updatesReleaseDetails').hidden=!latest||!(releaseName||notes||info?.prerelease||info?.download_name);$('updatesReleaseName').textContent=releaseName||'O que mudou';$('updatesReleaseNotes').textContent=notes||'Sem notas desta versão.';$('updatesPrerelease').hidden=info?.prerelease!==true;
 const downloadName=typeof info?.download_name==='string'?info.download_name.slice(0,256):'',size=Number.isFinite(info?.download_size)&&info.download_size>0?' · '+(info.download_size/1048576).toLocaleString('pt-BR',{maximumFractionDigits:1})+' MB':'';
 $('updatesDownloadInfo').hidden=!downloadName;$('updatesDownloadInfo').textContent=downloadName?'Instalador: '+downloadName+size:'';
 const blocked=!updates.native||updates.restricted||state.smoke;
 $('updatesCheck').disabled=blocked||!!updates.busy;$('updatesReleases').disabled=blocked||!!updates.busy;
 $('updatesDownload').hidden=!updateDownloadReady();$('updatesDownload').disabled=blocked||!!updates.busy||!updateDownloadReady();
 $('updatesCheckSpinner').hidden=!checking;$('updatesCheckLabel').textContent=checking?'Verificando…':status==='error'?'Tentar novamente':status==='available'?'Verificar novamente':'Verificar atualizações';
 $('updatesDownload').textContent=updates.busy==='download'?'Abrindo navegador…':'Baixar instalador';
 $('updatesCheck').classList.toggle('update-secondary',updateDownloadReady());
 $('updatesPreviewHint').hidden=!blocked;$('updatesNotice').hidden=!updates.notice;$('updatesNotice').textContent=updates.notice;$('updatesNotice').classList.toggle('is-error',updates.noticeError);
 $('updatesCheckedAt').textContent=Number.isFinite(info?.checked_at)&&info.checked_at>0?'Verificado às '+new Date(info.checked_at).toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'}):'Sem verificação automática';
}
async function initializeUpdates(){
 updates.native=!!window.__TAURI__;updates.restricted=state.smoke;renderUpdates();
 if(!updates.native)return;
 try{updates.info=await invoke('get_update_info');}catch{updates.notice='As informações de versão não estão disponíveis.';updates.noticeError=true;}
 renderUpdates();
}
async function updateAction(command){
 if(!updates.native||updates.restricted||state.smoke||updates.busy||command==='download_update'&&!updateDownloadReady())return;
 if(!['check_updates','open_releases','download_update'].includes(command))return;
 updates.busy=command==='check_updates'?'checking':command==='download_update'?'download':'opening';updates.notice='';updates.noticeError=false;renderUpdates();
 try{
  if(command==='check_updates')updates.info=await invoke(command);
  else{await invoke(command);updates.notice=command==='download_update'?'Continue o download no navegador. Depois, feche o AM8 Lab normalmente antes de executar o instalador.':'A página de Releases foi aberta no navegador.';}
 }catch(error){
  if(command==='check_updates')updates.info={...updates.info,status:'error',download_url:null};
  updates.notice=command==='check_updates'?'Não foi possível consultar o GitHub. Tente novamente ou abra Releases.':command==='download_update'?'Não foi possível iniciar o download. Verifique novamente ou abra Releases.':'Não foi possível abrir o navegador. Tente novamente.';updates.noticeError=true;
 }finally{updates.busy='';renderUpdates();}
}
$('updatesOpen').onclick=()=>{$('updatesDialog').showModal();renderUpdates();};
$('updatesClose').onclick=()=>$('updatesDialog').close();
$('updatesDialog').addEventListener('close',()=>$('updatesOpen').focus());
$('updatesCheck').onclick=()=>updateAction('check_updates');$('updatesReleases').onclick=()=>updateAction('open_releases');$('updatesDownload').onclick=()=>updateAction('download_update');
const fields={
 noise:[['threshold','Limiar',-70,-20,-40,'dB'],['ratio','Intensidade',1,10,4,''],['attack','Ataque',1,100,10,'ms'],['release','Liberação',50,1000,200,'ms']],
 pitch:[['pitch','Altura',-3,3,0,'st']],
 voicepro:[['pitch','Altura',80,120,100,'%'],['formant','Timbre',90,110,110,'%']],
 reverb:[['mix','Sinal reverberado',0,60,60,'%'],['room','Tamanho da sala',0,60,60,'%'],['damping','Amortecimento',0,100,50,'%'],['wet_gain','Volume do efeito',-18,-6,-6,'dB']],
 plate:[['decay','Decaimento',20,60,40,'%'],['predelay','Pré-atraso',0,2500,2500,'amostras'],['damping','Amortecimento',0,100,50,'%'],['wet_gain','Volume do efeito',-18,-6,-6,'dB']],
 compressor:[['threshold','Limiar',-40,-6,-18,'dB'],['ratio','Razão',1,6,2,':1'],['attack','Ataque',1,100,10,'ms'],['release','Liberação',50,1000,200,'ms']],feedback:[]};
const titles={noise:'Redução de ruído',pitch:'Tom da voz',voicepro:'Transformação de voz Pro',reverb:'Reverberação',compressor:'Compressor',feedback:'Supressão de microfonia'};
const hints={noise:'Atenua sons baixos entre as palavras.',pitch:'Pro: −3 a +3 semitons. Use com Transformação Pro e Reverb desligados.',voicepro:'Altura e timbre independentes. Desativa o Tom da voz. Use com Reverb desligado.',reverb:'Experimental: foram relatados estalos na reprodução. Desative Tom da voz e Transformação Pro antes de usar.',compressor:'Controla a dinâmica em uma única faixa.',feedback:'Experimental. Redução de microfonia ainda não validada.'};
const drafts={};
for(const name of Object.keys(titles))drafts[name]={enabled:false,mode:name==='pitch'?'pitchpro':name,values:Object.fromEntries(fields[name].map(f=>[f[0],f[4]]))};
const emptyEq=()=>({enabled:false,filters:Array.from({length:10},()=>({enabled:0,type:0,frequency:200,q:724,gain:0}))});
const emptyPlaybackEq=()=>({...emptyEq(),outputGain:0});
const equalizers={eq:emptyEq(),playbackeq:emptyPlaybackEq()};let eqDestination='eq';
const selectedEq=()=>equalizers[eqDestination];
const eqBoost=eq=>eq.filters.reduce((n,f)=>n+(f.enabled&&f.type<3?Math.max(0,f.gain):0),0);
const eqPreGain=eq=>(Object.hasOwn(eq,'outputGain')?eq.outputGain:0)-eqBoost(eq);
fields.pitchstandard=[['pitch','Altura',-12,12,0,'st']];
const fieldList=name=>fields[name==='reverb'?drafts.reverb.mode:name==='pitch'&&drafts.pitch.mode==='pitch'?'pitchstandard':name];
const display=(v,u)=>`${v>0&&['dB','st'].includes(u)?'+':''}${v}${u===':1'?u:u?' '+u:''}`;
function message(text,error=false){$('operationMessage').textContent=text;$('operationMessage').className=text?(error?'visible error':'visible'):'';}
const effectOrder=['noise','compressor','pitch','voicepro','reverb','feedback'];
function effectActive(name){
 const e=state.snapshot?.effects;
 if(!e)return false;
 return name==='pitch'?!!(e.pitch?.[0]||e.pitch_pro?.[0]):name==='reverb'?!!(e.reverb_native?.[0]||e.plate_native?.[0]):!!e[{voicepro:'voice_pro',feedback:'feedback_fine'}[name]||name]?.[0];
}
function effectConflict(name){
 if(!['pitch','voicepro','reverb'].includes(name)||!drafts[name].enabled)return '';
 const other=['pitch','voicepro','reverb'].find(x=>x!==name&&effectActive(x));
 return other?'Desative '+titles[other]+' no AM8 antes de aplicar este efeito.':'';
}
function refreshModules(){
 for(const card of document.querySelectorAll('[data-effect]')){
  const name=card.dataset.effect,dirty=state.dirty.has(name);
  const status=state.applying===name?'applying':state.failedEffect===name&&state.lastError?'error':dirty?'prepared':!state.connected?'unavailable':effectActive(name)?'active':'off';
  card.dataset.status=status;
  card.querySelector('.card-status').textContent={applying:'Aguardando confirmação…',error:'Falha · reconecte e confira a sessão',prepared:'Alterações preparadas',unavailable:'Conecte o AM8',active:'Ativo confirmado',off:'Desativado no AM8'}[status];
  const hint=card.querySelector('.effect-conflict'),conflict=effectConflict(name);hint.textContent=conflict;hint.hidden=!conflict;
 }
 const count=[...state.dirty].filter(x=>x!=='playbackeq').length;
 $('draftSummary').textContent=count?count+' módulo'+(count===1?'':'s')+' com alterações preparadas':'Nenhuma alteração preparada';
}
function mark(name){state.dirty.add(name);refreshModules();controls();}
function renderCards(){
 $('effectCards').innerHTML=effectOrder.map((name,index)=>{
  const title=titles[name];
  return `<article class="panel effect-module" data-effect="${name}"><div class="module-heading"><span class="module-number">${String(index+1).padStart(2,'0')}</span><div class="module-title"><h2>${title}</h2>${['feedback','reverb'].includes(name)?'<span class="badge amber">Experimental</span>':''}</div><label class="toggle"><input data-enable type="checkbox" aria-label="Ativar ${title}" ${drafts[name].enabled?'checked':''}></label></div><p class="subtle card-hint">${hints[name]}</p>${name==='pitch'||name==='reverb'?`<div class="segmented">${(name==='pitch'?[['pitch','Padrão'],['pitchpro','Pro']]:[['reverb','Sala'],['plate','Plate']]).map(([mode,label])=>`<button type="button" data-mode="${mode}" aria-pressed="${drafts[name].mode===mode}" class="${drafts[name].mode===mode?'selected':''}">${label}</button>`).join('')}</div>`:''}<div class="parameter-grid">${fieldList(name).map((f,i)=>`${name==='noise'&&i===2?'<details><summary>Controles avançados</summary>':''}<label class="field"><span>${f[1]}<output>${display(drafts[name].values[f[0]],f[5])}</output></span><input type="range" data-field="${f[0]}" min="${f[2]}" max="${f[3]}" step="1" value="${drafts[name].values[f[0]]}" aria-label="${f[1]}"></label>`).join('')}${name==='noise'?'</details>':''}</div><p class="effect-conflict" role="status" hidden></p><div class="card-footer"><span class="card-status">Prepare e aplique</span><button type="button" data-apply class="small-button">Aplicar</button></div></article>`;
 }).join('');
 document.querySelectorAll('[data-effect]').forEach(card=>{
  const name=card.dataset.effect;
  card.querySelector('[data-enable]').onchange=e=>{drafts[name].enabled=e.target.checked;mark(name);};
  card.querySelectorAll('[data-field]').forEach(input=>input.oninput=()=>{drafts[name].values[input.dataset.field]=Number(input.value);const f=fieldList(name).find(f=>f[0]===input.dataset.field);input.closest('label').querySelector('output').textContent=display(Number(input.value),f[5]);mark(name);});
  card.querySelectorAll('[data-mode]').forEach(button=>button.onclick=()=>{drafts[name].mode=button.dataset.mode;if(name==='reverb')drafts[name].values=Object.fromEntries(fieldList(name).map(f=>[f[0],f[4]]));mark(name);renderCards();controls();});
  card.querySelector('[data-apply]').onclick=()=>operation('apply_effect',{effect:['pitch','reverb'].includes(name)?drafts[name].mode:name,enabled:drafts[name].enabled,values:drafts[name].values},false,name);
 });refreshModules();
}
function updateNavigation(){
 for(const button of document.querySelectorAll('#navTabs [data-tab]')){
  const active=state.view==='eq'?(eqDestination==='playbackeq'?button.dataset.tab==='eq':button.dataset.tab==='effects'):button.dataset.tab===state.view;
  button.classList.toggle('nav-active',active);if(active)button.setAttribute('aria-current','page');else button.removeAttribute('aria-current');
 }
}
function switchTab(tab,destination){
 if(!['overview','effects','eq','profiles','capabilities'].includes(tab))return;
 if(destination&&destination!==eqDestination&&(state.busy||state.snapshot?.comparison))return;
 state.view=tab;if(destination)chooseEq(destination);
 document.querySelectorAll('.tab-content').forEach(x=>x.classList.toggle('active',x.id==='tab-'+tab));
 updateNavigation();$('mainContentArea').scrollTop=0;
}
document.querySelectorAll('[data-tab]').forEach(button=>button.onclick=()=>switchTab(button.dataset.tab,button.dataset.destination));
function controls(){
 const ready=state.connected&&!state.busy&&!state.snapshot?.comparison&&!state.snapshot?.legacy_pending&&!state.smoke;
 document.querySelectorAll('[data-effect] input,[data-effect] button,#eqFilters input,#eqFilters select,#eqQuick input,#eqEnabled,#eqApply,#eqHighpass,#eqLowpass,#eqFlat,#eqOutputGain,#eqOutputGainReset,#gainTest,#gainReduction').forEach(x=>x.disabled=!ready);
 document.querySelectorAll('[data-eq-destination]').forEach(x=>x.disabled=state.busy||!!state.snapshot?.comparison);
 $('restore').disabled=state.busy||!state.pending||state.smoke;
 $('btnCompareOriginal').disabled=state.busy||!state.connected||!state.pending||!!state.snapshot?.legacy_pending||state.smoke;
 for(const id of ['reconnect','reconnectBanner'])$(id).disabled=state.busy;
 $('compareText').textContent=state.snapshot?.comparison?'Voltar aos efeitos':'Comparar com original';
 $('bannerCompare').classList.toggle('hidden',!state.snapshot?.comparison);
 $('bannerDisconnected').classList.toggle('hidden',state.connected||state.busy);
 $('connLabel').textContent=state.busy?'Verificando / aplicando…':state.connected?'AM8 conectado por USB':'AM8 desconectado';
 $('connDot').style.background=state.connected?'#10b981':'#5a6172';
 document.querySelectorAll('[data-destination]').forEach(x=>x.disabled=state.busy||!!state.snapshot?.comparison);
 document.querySelectorAll('[data-effect]').forEach(card=>{if(effectConflict(card.dataset.effect))card.querySelector('[data-apply]').disabled=true;});refreshModules();
 if(!state.connected){$('firmware').textContent='Conecte o AM8 por USB';$('currentGainBadge').textContent='—';}
}
function snapshot(data,reset=false){
 state.snapshot=data;state.connected=true;state.pending=data.recovery_pending;if(reset)state.dirty.clear();
 const e=data.effects;
 for(const name of Object.keys(titles)){
  if(state.dirty.has(name))continue;const d=drafts[name];
  if(name==='pitch'){if(e.pitch_pro[0])d.mode='pitchpro';else if(e.pitch[0])d.mode='pitch';}
  if(name==='reverb'){if(e.plate_native[0])d.mode='plate';else if(e.reverb_native[0])d.mode='reverb';}
  const key={pitch:d.mode==='pitchpro'?'pitch_pro':'pitch',voicepro:'voice_pro',reverb:d.mode==='plate'?'plate_native':'reverb_native',feedback:'feedback_fine'}[name]||name,w=e[key];d.enabled=!!w[0];
  const v=name==='noise'?{threshold:w[1]/100,ratio:w[2],attack:w[3],release:w[4]}:name==='pitch'?{pitch:w[1]/10}:name==='voicepro'?{pitch:w[1],formant:w[2]}:name==='compressor'?{threshold:w[10]/100,ratio:w[14]/100,attack:w[18],release:w[22]}:name==='reverb'?(d.mode==='plate'?{decay:w[5],predelay:w[3],damping:w[6]/100,wet_gain:e.wet_route[2]/100}:{mix:w[2],room:w[4],damping:w[5],wet_gain:e.wet_route[2]/100}):{};
  d.values=Object.fromEntries(fieldList(name).map(f=>[f[0],(w[0]||state.pending)&&Number.isInteger(v[f[0]])&&v[f[0]]>=f[2]&&v[f[0]]<=f[3]?v[f[0]]:f[4]]));
 }
 for(const [effect,key] of [['eq','eq'],['playbackeq','playback_eq']])if(!state.dirty.has(effect)&&Array.isArray(e[key])){const w=e[key];const eq={enabled:!!w[0],filters:Array.from({length:10},(_,i)=>{const j=3+i*5;return {enabled:w[j],type:w[j+1],frequency:w[j+2],q:w[j+3],gain:w[j+4]/256};})};if(effect==='playbackeq')eq.outputGain=w[1]/256+eqBoost(eq);equalizers[effect]=eq;}
 $('firmware').textContent='Firmware '+data.firmware;$('currentGainBadge').textContent=data.gain_db.toFixed(1)+' dB';
 const names={noise:'Redução de ruído',pitch:'Tom padrão',pitch_pro:'Tom Pro',voice_pro:'Transformação Pro',reverb_native:'Reverb de sala',plate_native:'Reverb plate',feedback_fine:'Supressão de microfonia',compressor:'Compressor',eq:'Equalizador · microfone',playback_eq:'Equalizador · fones'};
 const active=Object.entries(names).filter(([key])=>e[key]?.[0]);$('activeCount').textContent=active.length?active.length+' efeitos ativos':'Sem efeitos ativos';$('activeEffects').replaceChildren();
 for(const [,name] of active){const row=document.createElement('div');row.className='active-effect';row.textContent=name;$('activeEffects').append(row);}if(!active.length)$('activeEffects').textContent='Sua sessão está no estado original.';
 renderCards();renderEq();controls();
}
async function operation(name,args={},reset=false,applied=null,automatic=false){
 if(state.busy)return;if(!automatic)state.lastError=null;state.busy=true;state.applying=applied;if(!automatic)state.failedEffect=null;controls();message('Aguardando confirmação do AM8…');
 try{const data=await invoke(name,args);if(applied)state.dirty.delete(applied);snapshot(data,reset);if(automatic&&state.lastError)message(state.lastError,true);else message(name==='restore_all'?'Sessão original restaurada.':name==='inspect'?'AM8 conectado e verificado.':'Ajuste confirmado no AM8.');}
 catch(error){state.failedEffect=applied;state.connected=false;try{state.pending=await invoke('recovery_pending');}catch{}state.lastError=String(error)+(state.pending?' Reconecte e restaure a sessão.':'');message(state.lastError,true);}
 finally{state.busy=false;state.applying=null;controls();}
}
$('btnCompareOriginal').onclick=() =>operation('compare_original');$('bannerCompare').querySelector('button').onclick=()=>operation('compare_original');
$('restore').onclick=()=>operation('restore_all',{},true);for(const id of ['reconnect','reconnectBanner'])$(id).onclick=()=>operation('inspect');
$('gainReduction').oninput=e=>$('gainReductionValue').textContent=e.target.value+' dB';$('gainTest').onclick=()=>operation('test_gain',{db:Number($('gainReduction').value)});

function renderEq(){
 const eq=selectedEq(),fones=eqDestination==='playbackeq';
 $('eqKicker').textContent=fones?'FONES / REPRODUÇÃO USB':'VOZ / CAPTURA USB';$('eqTitle').textContent=fones?'Seu áudio, do seu jeito.':'Ajuste cada detalhe da voz.';
 document.querySelectorAll('[data-eq-destination]').forEach(button=>{const selected=button.dataset.eqDestination===eqDestination;button.classList.toggle('selected',selected);button.setAttribute('aria-pressed',String(selected));});
 $('eqDescription').textContent=fones?'Ajuste o áudio do Windows enviado por USB aos fones no P2 do AM8.':'Dez filtros para ajustar sua voz captada pelo microfone.';
 $('eqValidation').hidden=!fones;$('eqDestinationHint').textContent=fones?'Corte de agudos confirmado neste AM8. As combinações usam limites conservadores. Escolha “Alto-falantes (fifine Microphone)” como saída do áudio. A conexão física do P2 não é detectada.':'Este EQ atua na voz enviada por USB. Os ajustes dos fones são independentes.';
 $('eqApply').textContent=fones?'Aplicar EQ dos fones':'Aplicar EQ do microfone';$('eqEnabled').setAttribute('aria-label',fones?'Ativar EQ dos fones':'Ativar EQ do microfone');
 $('eqOutputGainPanel').hidden=!fones;if(fones){$('eqOutputGain').value=eq.outputGain;$('eqOutputGainValue').textContent=display(eq.outputGain,'dB');}
 $('eqEnabled').checked=eq.enabled;for(const [i,id] of ['quickBass','quickMid','quickTreble'].entries()){$(id).value=eq.filters[i].gain;$(id+'Val').textContent=display(eq.filters[i].gain,'dB');}$('eqFilters').innerHTML='<div class="eq-row eq-heading"><span>Filtro</span><span>Tipo</span><span>Frequência · Hz</span><span>Ganho · dB</span><span>Q</span></div>'+eq.filters.map((f,i)=>`<div class="eq-row"><label class="switch-label"><input type="checkbox" data-index="${i}" data-eq="enabled" ${f.enabled?'checked':''}> ${i+1}</label><select data-index="${i}" data-eq="type" aria-label="Tipo do filtro ${i+1}">${['Pico','Graves shelf','Agudos shelf','Passa-baixas','Passa-altas'].map((n,j)=>`<option value="${j}" ${f.type===j?'selected':''}>${n}</option>`).join('')}</select><input data-index="${i}" data-eq="frequency" type="number" min="20" max="16000" step="1" value="${f.frequency}" aria-label="Frequência do filtro ${i+1}"><input data-index="${i}" data-eq="gain" type="number" min="-6" max="6" step="1" value="${f.gain}" aria-label="Ganho do filtro ${i+1}"><input data-index="${i}" data-eq="q" type="number" min="0.25" max="8" step="0.01" value="${(f.q/1024).toFixed(2)}" aria-label="Q do filtro ${i+1}"></div>`).join('');
  $('eqFilters').querySelectorAll('[data-eq]').forEach(input=>input.onchange=()=>{const key=input.dataset.eq;let v=key==='enabled'?Number(input.checked):Number(input.value);if(key==='q')v=Math.round(v*1024);eq.filters[Number(input.dataset.index)][key]=v;markEq();drawCurve();});
 $('eqStatus').textContent=state.dirty.has(eqDestination)?'Alterações preparadas':state.snapshot?.effects?.[fones?'playback_eq':'eq']?'Valores confirmados':'Prepare e aplique';drawCurve();
}
function markEq(){mark(eqDestination);$('eqStatus').textContent='Alterações preparadas';}
function eqValues(){const eq=selectedEq(),v={};eq.filters.forEach((f,i)=>{for(const key of ['enabled','type','frequency','q','gain'])v[`f${i}_${key}`]=f[key];});if(eqDestination==='playbackeq')v.output_gain=eq.outputGain;return v;}
const eqRequest=()=>({effect:eqDestination,enabled:selectedEq().enabled,values:eqValues()});
function chooseEq(destination){if(!Object.hasOwn(equalizers,destination))return;eqDestination=destination;renderEq();controls();updateNavigation();}
document.querySelectorAll('[data-eq-destination]').forEach(button=>button.onclick=()=>chooseEq(button.dataset.eqDestination));
$('eqEnabled').onchange=e=>{selectedEq().enabled=e.target.checked;markEq();};$('eqApply').onclick=()=>operation('apply_effect',eqRequest(),false,eqDestination);
$('eqOutputGain').oninput=()=>{if(eqDestination!=='playbackeq')return;const gain=Number($('eqOutputGain').value);if(!Number.isInteger(gain)||gain<0||gain>18)return;selectedEq().outputGain=gain;markEq();$('eqOutputGainValue').textContent=display(gain,'dB');drawCurve();};
$('eqOutputGainReset').onclick=()=>{if(eqDestination!=='playbackeq')return;selectedEq().outputGain=0;markEq();renderEq();controls();};
for(const [id,type,hz] of [['eqHighpass',4,400],['eqLowpass',3,2500],['eqFlat',0,200]])$(id).onclick=()=>{const eq=selectedEq();eq.filters.forEach(f=>f.enabled=0);eq.filters[0]={enabled:id==='eqFlat'?0:1,type,frequency:hz,q:724,gain:0};eq.enabled=id!=='eqFlat'||eqDestination==='playbackeq'&&eq.outputGain>0;markEq();renderEq();controls();};
// Standard biquad response estimate at 44.1 kHz; no audio capture or processing.
function filterDb(f,hz){
 if(!f.enabled||!Number.isFinite(f.frequency)||!Number.isFinite(f.q)||f.q<=0)return 0;
 const w=2*Math.PI*f.frequency/44100,c=Math.cos(w),s=Math.sin(w),A=10**(f.gain/40),alpha=s/(2*f.q/1024),beta=2*Math.sqrt(A)*alpha;let b,a;
 if(f.type===0){b=[1+alpha*A,-2*c,1-alpha*A];a=[1+alpha/A,-2*c,1-alpha/A];}
 else if(f.type===3){b=[(1-c)/2,1-c,(1-c)/2];a=[1+alpha,-2*c,1-alpha];}
 else if(f.type===4){b=[(1+c)/2,-(1+c),(1+c)/2];a=[1+alpha,-2*c,1-alpha];}
 else if(f.type===1){b=[A*((A+1)-(A-1)*c+beta),2*A*((A-1)-(A+1)*c),A*((A+1)-(A-1)*c-beta)];a=[(A+1)+(A-1)*c+beta,-2*((A-1)+(A+1)*c),(A+1)+(A-1)*c-beta];}
 else{b=[A*((A+1)+(A-1)*c+beta),-2*A*((A-1)+(A+1)*c),A*((A+1)+(A-1)*c-beta)];a=[(A+1)-(A-1)*c+beta,2*((A-1)-(A+1)*c),(A+1)-(A-1)*c-beta];}
 const x=2*Math.PI*hz/44100,mag=v=>Math.hypot(v[0]+v[1]*Math.cos(x)+v[2]*Math.cos(2*x),v[1]*Math.sin(x)+v[2]*Math.sin(2*x));return 20*Math.log10(Math.max(1e-8,mag(b)/mag(a)));
}
function drawCurve(){
 const eq=selectedEq();
 let grid='';for(const y of [20,60,100,140,180])grid+=`<path d="M0 ${y}H920" stroke="#2d3139"/>`;for(const hz of [100,1000,10000]){const x=Math.log(hz/20)/Math.log(800)*920;grid+=`<path d="M${x} 0V200" stroke="#2d3139" stroke-dasharray="3 5"/>`;}
 const boost=eqBoost(eq),preGain=eqPreGain(eq);
 $('eqCompensationValue').textContent=display(-boost,'dB');$('eqPreGainValue').textContent=display(preGain,'dB');
 const pts=Array.from({length:230},(_,i)=>{const db=eq.filters.reduce((n,f)=>n+filterDb(f,20*(800**(i/229))),preGain);return `${i?'L':'M'}${i*920/229},${100-Math.max(-24,Math.min(24,db))*3.3}`;}).join(' ');
 $('eqCurve').innerHTML=grid+`<path d="M0 100H920" stroke="#3d424e" stroke-dasharray="4 5"/><path d="${pts}" fill="none" stroke="#d0bcff" stroke-width="2"/>`;
}
const profileKey='am8lab-profiles-v1',profileNameLimit=128;let profiles=[];
const isRecord=value=>value!==null&&typeof value==='object'&&!Array.isArray(value);
const hasKeys=(value,keys)=>isRecord(value)&&Object.keys(value).length===keys.length&&keys.every(key=>Object.hasOwn(value,key));
function eqProfileValid(value,playback=false){
 const knownKeys=hasKeys(value,['enabled','filters'])||playback&&hasKeys(value,['enabled','filters','outputGain']);
 return knownKeys&&(!Object.hasOwn(value,'outputGain')||Number.isInteger(value.outputGain)&&value.outputGain>=0&&value.outputGain<=18)&&typeof value.enabled==='boolean'&&Array.isArray(value.filters)&&value.filters.length===10&&value.filters.every(f=>hasKeys(f,['enabled','type','frequency','q','gain'])&&Object.entries({enabled:[0,1],type:[0,4],frequency:[20,16000],q:[256,8192],gain:[-6,6]}).every(([k,[lo,hi]])=>Number.isInteger(f[k])&&f[k]>=lo&&f[k]<=hi));
}
function profileValid(p){
 if(!(hasKeys(p,['name','drafts','eq'])||hasKeys(p,['name','drafts','eq','playbackEq']))||typeof p.name!=='string'||!p.name.trim()||p.name.length>profileNameLimit)return false;
 if(!hasKeys(p.drafts,Object.keys(titles))||!eqProfileValid(p.eq)||Object.hasOwn(p,'playbackEq')&&!eqProfileValid(p.playbackEq,true))return false;
 for(const name of Object.keys(titles)){const d=p.drafts[name];if(!hasKeys(d,['enabled','mode','values'])||typeof d.enabled!=='boolean'||!isRecord(d.values))return false;
  if(name==='pitch'&&!['pitch','pitchpro'].includes(d.mode)||name==='reverb'&&!['reverb','plate'].includes(d.mode))return false;
   if(!['pitch','reverb'].includes(name)&&d.mode!==name)return false;
   const list=fields[name==='reverb'?d.mode:name==='pitch'&&d.mode==='pitch'?'pitchstandard':name];if(!hasKeys(d.values,list.map(f=>f[0]))||list.some(f=>!Number.isInteger(d.values[f[0]])||d.values[f[0]]<f[2]||d.values[f[0]]>f[3]))return false;
 }
 return true;
}
try{const raw=localStorage.getItem(profileKey)||'[]';if(raw.length<200000){const parsed=JSON.parse(raw);if(Array.isArray(parsed))profiles=parsed.filter(profileValid).slice(0,20);}}catch{}
function loadProfile(p){
 if(!profileValid(p)){message('Perfil inválido; nenhum ajuste enviado.',true);return false;}
 for(const name of Object.keys(titles)){drafts[name]=structuredClone(p.drafts[name]);state.dirty.add(name);}equalizers.eq=structuredClone(p.eq);state.dirty.add('eq');
 if(Object.hasOwn(p,'playbackEq')){equalizers.playbackeq={...structuredClone(p.playbackEq),outputGain:p.playbackEq.outputGain??0};state.dirty.add('playbackeq');}
 renderCards();renderEq();controls();message(Object.hasOwn(p,'playbackEq')?'Perfil carregado. Aplique cada efeito e cada EQ desejado.':'Perfil antigo carregado. Os controles dos fones foram mantidos. Aplique os efeitos desejados.');return true;
}
function renderProfiles(){
 profiles=profiles.filter(profileValid).slice(0,20);$('profileCount').textContent=profiles.length+' / 20 perfis';
 $('profileList').replaceChildren();for(const [index,p] of profiles.entries()){
  const row=document.createElement('div');row.className='panel profile-row';const label=document.createElement('strong');label.textContent=typeof p.name==='string'?p.name:'Perfil';row.append(label);
  const actions=document.createElement('div');actions.className='flex gap-2';const load=document.createElement('button');load.className='small-button';load.textContent='Carregar controles';load.onclick=()=>loadProfile(p);const remove=document.createElement('button');remove.className='small-button';remove.textContent='Excluir';remove.onclick=()=>{profiles.splice(index,1);try{localStorage.setItem(profileKey,JSON.stringify(profiles));}catch{}renderProfiles();};actions.append(load,remove);row.append(actions);$('profileList').append(row);
 }if(!profiles.length)$('profileList').textContent='Nenhum perfil salvo ainda.';
}
$('saveProfile').onclick=()=>{const name=$('profileName').value.trim();if(!name||name.length>profileNameLimit||profiles.length>=20){message('Informe um nome de até 128 caracteres. Limite: 20 perfis.',true);return;}const p={name,drafts:structuredClone(drafts),eq:structuredClone(equalizers.eq),playbackEq:structuredClone(equalizers.playbackeq)};if(!profileValid(p)){message('Revise os valores dos controles antes de salvar.',true);return;}try{profiles.push(p);localStorage.setItem(profileKey,JSON.stringify(profiles));$('profileName').value='';renderProfiles();message('Perfil salvo neste computador.');}catch{message('Não foi possível salvar o perfil.',true);}};

const meters={voice:{raw:0,view:0,peak:0,last:0},playback:{raw:0,view:0,peak:0,last:0}};let lastFrame=performance.now();
const meterVisual=raw=>raw<=0?0:Math.max(0,Math.min(1,(Math.log10(Math.max(raw,32))-Math.log10(32))/(Math.log10(32767)-Math.log10(32))));
function animate(now){
 const dt=Math.min(100,now-lastFrame);lastFrame=now;
 for(const [key,m] of Object.entries(meters)){
  if(!state.connected||now-m.last>400)m.raw=0;
  const target=meterVisual(m.raw);m.view=target>=m.view||matchMedia('(prefers-reduced-motion: reduce)').matches?target:m.view+(target-m.view)*(1-Math.exp(-dt/45));
  m.peak=m.view;
  for(const stem of key==='voice'?['mic','headerMic']:['playback','headerPlayback']){
   $(stem+'MeterFill').style.width=(m.view*100).toFixed(2)+'%';$(stem+'MeterPeak').style.width='2px';$(stem+'MeterPeak').style.left=(m.peak*100).toFixed(2)+'%';$(stem+'MeterVal').textContent=(m.raw/32767*100).toFixed(1)+'%';
  }
 }requestAnimationFrame(animate);
}
async function start(){
 renderCards();renderEq();renderProfiles();switchTab('overview');controls();renderUpdates();requestAnimationFrame(animate);
 if(!window.__TAURI__){message('Prévia do design. O USB está disponível no aplicativo.',true);return;}
 state.smoke=await invoke('smoke_mode');
 if(await invoke('preview_mode')){state.smoke=true;await initializeUpdates();controls();message('Prévia do design — sem comunicação USB.');return;}
 await initializeUpdates();
 await window.__TAURI__.event.listen('operation-error',e=>{state.busy=false;message(e.payload,true);controls();});
 await window.__TAURI__.event.listen('closing-restore',()=>{state.busy=true;message('Restaurando os valores originais para fechar…');controls();});
 await window.__TAURI__.event.listen('meters',e=>{const d=e.payload,now=performance.now();if(d.sampled_at&&Date.now()-d.sampled_at>400)return;for(const key of ['voice','playback'])if(Number.isInteger(d[key])&&d[key]>=0&&d[key]<=32767){meters[key].raw=d[key];meters[key].last=now;}$('outputStatusText').textContent=d.output?'Saída padrão: '+d.output:'Saída padrão indisponível.';if(!d.connected){state.connected=false;controls();if(d.error&&!state.busy)message(d.error,true);}else if(!state.connected&&!state.busy)operation('inspect',{},false,null,true);});
 if(state.smoke){
  message('Verificação da interface — sem comunicação USB.');
  const sample={name:'Teste',drafts:structuredClone(drafts),eq:structuredClone(equalizers.eq),playbackEq:structuredClone(equalizers.playbackeq)};
  const {playbackEq:ignoredPlayback,...oldSample}=sample;
  const oldPlaybackSample={...structuredClone(sample),playbackEq:{enabled:sample.playbackEq.enabled,filters:structuredClone(sample.playbackEq.filters)}};
  let passed=document.querySelectorAll('[data-effect]').length===6&&$('eqFilters').querySelectorAll('[data-eq="type"]').length===10&&profileValid(sample)&&profileValid(oldSample)&&profileValid(oldPlaybackSample)&&$('eqOutputGain').min==='0'&&$('eqOutputGain').max==='18'&&$('eqOutputGain').step==='1'&&$('eqOutputGainPanel').hidden;
  for(const malformed of [null,[],0,'perfil',{}, {...sample,name:'x'.repeat(profileNameLimit+1)}, {...sample,drafts:[]}, {...sample,eq:{enabled:false,filters:Array(10).fill(null)}}])passed=passed&&!profileValid(malformed);
  const outOfRange=structuredClone(sample);outOfRange.eq.filters[0].gain=7;passed=passed&&!profileValid(outOfRange);
  const invalidValues=structuredClone(sample);invalidValues.drafts.noise.values=null;passed=passed&&!profileValid(invalidValues);
  for(const malformedPlayback of [null,[],{}, {enabled:1,filters:sample.playbackEq.filters}, {enabled:false,filters:Array(10).fill(null)}, {...sample.playbackEq,extra:0}])passed=passed&&!profileValid({...sample,playbackEq:malformedPlayback});
  for(const [field,value] of [['enabled',2],['type',5],['frequency',16001],['q',8193],['gain',7],['gain',1.5]]){const invalid=structuredClone(sample);invalid.playbackEq.filters[0][field]=value;passed=passed&&!profileValid(invalid);}
  for(const outputGain of [-1,19,0.5,'3',null,undefined,NaN,Infinity])passed=passed&&!profileValid({...sample,playbackEq:{...sample.playbackEq,outputGain}});
  for(const outputGain of [0,18])passed=passed&&profileValid({...sample,playbackEq:{...sample.playbackEq,outputGain}});
  passed=passed&&!profileValid({...sample,eq:{...sample.eq,outputGain:0}});
  const originalEqualizers=structuredClone(equalizers),originalDestination=eqDestination,originalDirty=new Set(state.dirty),originalDrafts=structuredClone(drafts),originalSnapshot=state.snapshot,originalConnected=state.connected,originalPending=state.pending;state.dirty.clear();
  $('eqDestinationMic').click();$('quickBass').value='-2';$('quickBass').oninput();const preparedMic=JSON.stringify(equalizers.eq);
  passed=passed&&eqRequest().effect==='eq'&&eqRequest().values.f0_gain===-2&&Object.keys(eqRequest().values).length===50&&!Object.hasOwn(eqRequest().values,'output_gain')&&state.dirty.has('eq')&&!state.dirty.has('playbackeq');
  $('eqDestinationFones').click();$('eqOutputGain').value='12';$('eqOutputGain').oninput();$('quickBass').value='3';$('quickBass').oninput();const preparedPlayback=JSON.stringify(equalizers.playbackeq);
  passed=passed&&eqRequest().effect==='playbackeq'&&eqRequest().values.f0_gain===3&&Object.keys(eqRequest().values).length===51&&eqRequest().values.output_gain===12&&$('eqCompensationValue').textContent==='-3 dB'&&$('eqPreGainValue').textContent==='+9 dB'&&JSON.stringify(equalizers.eq)===preparedMic&&state.dirty.has('playbackeq')&&!$('eqValidation').hidden&&!$('eqOutputGainPanel').hidden;
  $('eqDestinationMic').click();passed=passed&&$('quickBass').value==='-2'&&eqRequest().effect==='eq'&&JSON.stringify(equalizers.playbackeq)===preparedPlayback&&$('eqValidation').hidden&&$('eqOutputGainPanel').hidden;
  loadProfile(oldSample);passed=passed&&JSON.stringify(equalizers.playbackeq)===preparedPlayback;
  loadProfile(oldPlaybackSample);passed=passed&&equalizers.playbackeq.outputGain===0;
  loadProfile(sample);passed=passed&&JSON.stringify(equalizers.eq)===JSON.stringify(sample.eq)&&JSON.stringify(equalizers.playbackeq)===JSON.stringify(sample.playbackEq);
  chooseEq('playbackeq');$('eqOutputGain').value='18';$('eqOutputGain').oninput();$('eqFlat').onclick();passed=passed&&equalizers.playbackeq.outputGain===18&&equalizers.playbackeq.enabled&&eqRequest().values.output_gain===18&&eqPreGain(selectedEq())===18;
  $('eqOutputGainReset').onclick();passed=passed&&equalizers.playbackeq.outputGain===0&&$('eqOutputGain').value==='0'&&eqRequest().values.output_gain===0;
  const nativeEq=Array.from({length:53},(_,i)=>i>=3&&((i-3)%5===2)?200:i>=3&&((i-3)%5===3)?724:0),nativePlayback=[...nativeEq];nativePlayback[0]=1;nativePlayback[1]=10*256;nativePlayback[3]=1;nativePlayback[7]=2*256;nativePlayback[8]=1;nativePlayback[9]=4;nativePlayback[12]=6*256;
  snapshot({firmware:'TEST',gain_db:0,recovery_pending:true,effects:{noise:[0,-4000,4,10,200],pitch:[0,0],pitch_pro:[0,0],voice_pro:[0,100,100],reverb_native:[0,0,60,0,60,50],plate_native:[0,0,0,2500,0,40,5000],wet_route:[0,0,-600],feedback_fine:[0],compressor:Array(23).fill(0),eq:nativeEq,playback_eq:nativePlayback}},true);
  passed=passed&&equalizers.playbackeq.outputGain===12&&eqPreGain(equalizers.playbackeq)===10&&$('eqOutputGain').value==='12'&&$('eqPreGainValue').textContent==='+10 dB'&&Object.keys(equalizers.eq).length===2;
  for(const name of Object.keys(titles))drafts[name]=originalDrafts[name];Object.assign(equalizers,originalEqualizers);state.dirty=originalDirty;state.snapshot=originalSnapshot;state.connected=originalConnected;state.pending=originalPending;chooseEq(originalDestination);renderCards();controls();
  const originalProfiles=profiles;profiles=[null,{...sample,name:'<img src=x onerror="alert(1)">'}];renderProfiles();
  passed=passed&&profiles.length===1&&$('profileList').querySelector('strong').textContent===profiles[0].name&&!$('profileList').querySelector('img');profiles=originalProfiles;renderProfiles();
  for(const tab of ['effects','eq','profiles','capabilities','overview']){$('nav-tab-'+tab).click();passed=passed&&$('tab-'+tab).classList.contains('active');}
  passed=passed&&!$('eqCurve').innerHTML.includes('NaN');
  const originalUpdates={...updates},originalSmoke=state.smoke,originalBusy=state.busy,originalUpdatesConnected=state.connected;
  const updateSample={current_version:'0.6.7',latest_version:'0.6.8',status:'available',download_url:'https://github.com/NightXXT/am8lab/releases/download/v0.6.8/AM8-Lab-Setup-v0.6.8.exe',release_name:'<img src=x onerror="alert(1)">',release_notes:'<script>alert(1)</script>\nTexto da versão',checked_at:1};
  updates.info=updateSample;renderUpdates();passed=passed&&$('updatesVersion').textContent==='Versão 0.6.7'&&!$('updatesDownload').hidden&&$('updatesDownload').disabled&&$('updatesCheck').disabled&&!$('updatesDot').hidden&&$('updatesReleaseName').textContent===updateSample.release_name&&$('updatesReleaseNotes').textContent===updateSample.release_notes&&!$('updatesReleaseDetails').querySelector('img,script');
  $('updatesOpen').click();passed=passed&&$('updatesDialog').open;$('updatesClose').click();passed=passed&&!$('updatesDialog').open;
  await updateAction('check_updates');await updateAction('download_update');await updateAction('open_releases');
  state.smoke=false;state.busy=true;state.connected=false;updates.restricted=false;updates.native=true;renderUpdates();passed=passed&&!$('updatesDownload').disabled&&!$('updatesCheck').disabled&&!$('updatesReleases').disabled;
  updates.busy='checking';renderUpdates();passed=passed&&$('updatesCheck').disabled&&$('updatesDownload').disabled&&$('updatesStateTitle').textContent==='Verificando atualizações…'&&!$('updatesCheckSpinner').hidden&&state.busy;
  updates.busy='';for(const status of ['idle','current','ahead','no_release','unavailable','error']){updates.info={...updateSample,status,download_url:null};renderUpdates();passed=passed&&$('updatesDownload').hidden&&$('updatesDot').hidden&&$('updatesStateTitle').textContent===updateStatuses[status][0]&&!$('updatesCheck').disabled&&!$('updatesReleases').disabled;}
  updates.info={...updateSample,latest_version:null};renderUpdates();passed=passed&&$('updatesDownload').hidden;
  updates.info={...updateSample,download_url:null};renderUpdates();passed=passed&&$('updatesDownload').hidden;
  state.smoke=originalSmoke;state.busy=originalBusy;state.connected=originalUpdatesConnected;Object.assign(updates,originalUpdates);renderUpdates();
  await invoke('finish_smoke',{passed});return;
 }
 await operation('inspect');
}
start().catch(error=>message(String(error),true));

for(const id of ['quickBass','quickMid','quickTreble'])$(id).oninput=()=>{
 const eq=selectedEq();
 const gains=['quickBass','quickMid','quickTreble'].map(x=>Number($(x).value));
 eq.filters.forEach(f=>f.enabled=0);
 for(const [i,[type,hz]] of [[1,150],[0,1200],[2,5000]].entries())eq.filters[i]={enabled:1,type,frequency:hz,q:724,gain:gains[i]};
 eq.enabled=true;markEq();renderEq();controls();
};
